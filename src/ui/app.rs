use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    future,
    rc::Rc,
    sync::Arc,
};

use iced::{
    widget::{column, container, horizontal_space, row, text, vertical_space, Column},
    Alignment, Application, Command, Element, Length, Theme,
};
use iced_native::command::Action;
use once_cell::sync::Lazy;
use smol::future::FutureExt;

pub static APP_NAME: Lazy<Arc<str>> = Lazy::new(|| "J-Station Controller".into());

use crate::jstation::{
    self,
    data::{dsp, Program, ProgramNumber},
};
use crate::midi;
use crate::ui;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Couldn't find J-Station")]
    JStationNotFound,
    #[error("J-Station error: {}", .0)]
    JStation(#[from] jstation::Error),
    #[error("Unexpected program received {}, expected {}", .received, .expected)]
    UnexpectedProgramNumber {
        received: ProgramNumber,
        expected: ProgramNumber,
    },
    #[error("Unexpected program received {}", .0)]
    UnexpectedProgram(ProgramNumber),
}

pub struct App {
    jstation: ui::jstation::Interface,
    dsp: dsp::Dsp,
    programs: BTreeMap<ProgramNumber, Program>,
    program_indices: VecDeque<u8>,
    cur_prog: Option<ProgramNumber>,

    show_midi_modal: bool,
    ports: Rc<RefCell<ui::midi::Ports>>,
    scanner_ctx: Option<midi::scanner::Context>,

    show_utility_settings: bool,

    use_dark_them: bool,
    output_text: String,
}

impl App {
    fn show_error(&mut self, err: &dyn std::error::Error) {
        log::error!("{err}");
        self.output_text = err.to_string();
    }

    fn handle_jstation_event(
        &mut self,
        res: Result<jstation::Message, jstation::Error>,
    ) -> Result<Command<Message>, Error> {
        use jstation::Message::*;
        match res {
            Ok(SysEx(sysex)) => {
                use jstation::Procedure::*;
                match Arc::try_unwrap(sysex).unwrap().proc {
                    WhoAmIResp(resp) => {
                        self.programs.clear();

                        self.jstation.have_who_am_i_resp(resp).map_err(|err| {
                            self.jstation.clear();
                            self.ports.borrow_mut().set_disconnected();

                            err
                        })?;

                        self.output_text = "Found J-Station".to_string();

                        let (port_in, port_out) =
                            self.jstation.connected_ports().expect("Not connected");
                        self.ports.borrow_mut().set_ports(port_in, port_out);

                        return Ok(Command::single(Action::Future(
                            future::ready(Message::HideModal).boxed(),
                        )));
                    }
                    UtilitySettingsResp(resp) => {
                        self.dsp.utility_settings = resp.try_into()?;

                        // FIXME handle ui consequence of the error
                        self.jstation.bank_dump()?;
                    }
                    ProgramIndicesResp(resp) => {
                        self.program_indices = VecDeque::from_iter(resp.indices.iter().cloned());
                    }
                    OneProgramResp(resp) => {
                        let prog_nb = resp.prog.nb();

                        if let Some(indice) = self.program_indices.pop_front() {
                            if prog_nb.as_u8() == indice {
                                self.programs.insert(prog_nb, resp.prog);
                            } else {
                                self.show_error(&Error::UnexpectedProgramNumber {
                                    received: prog_nb,
                                    expected: ProgramNumber::from(indice),
                                });
                            }
                        } else {
                            self.show_error(&Error::UnexpectedProgram(prog_nb));
                        }
                    }
                    EndBankDumpResp(_) => {
                        self.jstation.program_update_req()?;
                    }
                    ProgramUpdateResp(resp) => {
                        self.dsp.set_raw(resp.prog_data.data())?;
                        // FIXME figure out the selected prog_nb from its data & name
                        //self.cur_prog = Some(prog_nb);
                    }
                    other => {
                        log::debug!("Unhandled {other:?}");
                    }
                }
            }
            Ok(ChannelVoice(cv)) => {
                use jstation::channel_voice::Message::*;
                use jstation::data::CCParameterSetter;
                match cv.msg {
                    CC(cc) => match self.dsp.set_cc(cc) {
                        Ok(Some(param)) => log::debug!("Updated {param:?} from {cc:?}"),
                        Ok(None) => log::debug!("Unchanged param for {cc:?}"),
                        Err(err) => log::warn!("{err}"),
                    },
                    ProgramChange(prog_nb) => {
                        log::debug!("Requested {prog_nb:?}");
                        if let Some(prog) = self.programs.get(&prog_nb) {
                            if let Err(err) = self.dsp.set_raw(prog.data()) {
                                // FIXME currently an Err is returned setting
                                // an inactive Effect/Regen, but the program
                                // is selected on device.
                                log::debug!("{err}");
                            }
                            self.cur_prog = Some(prog_nb);
                        } else {
                            // FIXME could be an error
                            log::warn!("Requested unknown {prog_nb:?}");
                        }
                    }
                }
            }
            Err(err) if err.is_handshake_timeout() => {
                if let Some(scanner_ctx) = self.scanner_ctx.take() {
                    self.scanner_ctx = self.jstation.scan_next(scanner_ctx);
                }

                if self.scanner_ctx.is_none() {
                    self.jstation.clear();
                    self.ports.borrow_mut().set_disconnected();
                    self.show_midi_modal = true;

                    return Err(Error::JStationNotFound);
                }
            }
            Err(err) => {
                // Switch to true to panic on first error
                if false {
                    panic!("{err}");
                } else {
                    Err(err)?;
                }
            }
        }

        Ok(Command::none())
    }
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        let mut output_text = " ".to_string();

        let mut jstation = ui::jstation::Interface::new();
        let mut ports = ui::midi::Ports::default();

        match jstation.refresh() {
            Ok(()) => ports.update_from(jstation.iface()),
            Err(err) => {
                // FIXME set a flag to indicate the application can't be used as is
                let msg = format!("Midi ports not found: {err}");
                log::error!("{msg}");
                output_text = msg;
            }
        }

        let app = App {
            jstation,

            dsp: dsp::Dsp::default(),
            programs: BTreeMap::new(),
            program_indices: VecDeque::new(),
            cur_prog: None,

            show_midi_modal: false,
            ports: RefCell::new(ports).into(),
            scanner_ctx: None,

            show_utility_settings: false,

            use_dark_them: true,
            output_text,
        };

        (
            app,
            Command::single(Action::Future(future::ready(Message::StartScan).boxed())),
        )
    }

    fn title(&self) -> String {
        APP_NAME.to_string()
    }

    fn theme(&self) -> Self::Theme {
        if self.use_dark_them {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn update(&mut self, event: Message) -> Command<Message> {
        use Message::*;
        match event {
            JStation(res) => match self.handle_jstation_event(res) {
                Ok(cmd) => return cmd,
                Err(err) => self.show_error(&err),
            },
            Parameter(param) => {
                use jstation::data::{CCParameter, ParameterSetter};

                if self.dsp.set(param).is_some() {
                    log::debug!("Set {param:?}");
                }

                if let Some(cc) = param.to_cc() {
                    // FIXME handle the error
                    dbg!(&cc);
                    let _ = self.jstation.send_cc(cc);
                } else {
                    log::error!("No CC for {:?}", param);
                }
            }
            Midi(ui::midi::Selection { port_in, port_out }) => {
                use midi::Scannable;
                if let Err(err) = self.jstation.connect(port_in, port_out) {
                    self.jstation.clear();
                    self.ports.borrow_mut().set_disconnected();
                    self.show_error(&err);
                }
            }
            SelectProgram(prog_nb) => {
                if let Some(prog) = self.programs.get(&prog_nb) {
                    match self
                        .dsp
                        .set_raw(prog.data())
                        .and_then(|()| self.jstation.change_program(prog_nb))
                    {
                        Ok(()) => self.cur_prog = Some(prog_nb),
                        Err(err) => self.show_error(&err),
                    }
                } else {
                    // FIXME could be an error
                    log::warn!("Requested unknown {prog_nb:?}");
                }
            }
            StartScan => {
                log::debug!("Scanning Midi ports for J-Station");
                self.scanner_ctx = self.jstation.start_scan();

                if self.scanner_ctx.is_none() {
                    self.output_text = "Couldn't scan for J-Station".to_string();
                }
            }
            UtilitySettings(settings) => {
                log::debug!("Got UtilitySettings UI update");
                self.dsp.utility_settings = settings;
                // FIXME send message to device
            }
            UseDarkTheme(use_dark) => self.use_dark_them = use_dark,
            ShowMidiConnection => self.show_midi_modal = true,
            ShowUtilitySettings => self.show_utility_settings = true,
            HideModal => {
                self.show_midi_modal = false;
                self.show_utility_settings = false;
            }
        }

        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        self.jstation.subscription().map(Message::JStation)
    }

    fn view(&self) -> Element<Message> {
        use jstation::data::BoolParameter;
        use Message::*;

        let effect_post = self.dsp.effect.post;

        let content: Element<_> = if self.show_midi_modal {
            ui::modal(
                column![
                    ui::midi::Panel::new(self.ports.clone(), Midi),
                    vertical_space(Length::Units(20)),
                    ui::button("Scan").on_press(StartScan),
                ]
                .align_items(Alignment::End),
                HideModal,
            )
            .into()
        } else if self.show_utility_settings {
            ui::modal(
                ui::utility_settings::Panel::new(self.dsp.utility_settings, Message::from),
                HideModal,
            )
            .into()
        } else {
            let mut dspes = column![
                ui::compressor::Panel::new(self.dsp.compressor),
                ui::wah_expr::Panel::new(self.dsp.wah_expr),
            ]
            .spacing(10);

            if !effect_post.is_true() {
                dspes = dspes.push(ui::effect::Panel::new(self.dsp.effect));
            }

            dspes = dspes.push(ui::amp::Panel::new(self.dsp.amp));

            dspes = dspes.push(row![
                ui::dsp_keep_width(ui::cabinet::Panel::new(self.dsp.cabinet)),
                horizontal_space(Length::Units(10)),
                ui::dsp_keep_width(ui::noise_gate::Panel::new(self.dsp.noise_gate)),
            ]);

            if effect_post.is_true() {
                dspes = dspes.push(ui::effect::Panel::new(self.dsp.effect));
            }

            dspes = dspes.push(ui::delay::Panel::new(self.dsp.delay));
            dspes = dspes.push(ui::reverb::Panel::new(self.dsp.reverb));

            let progs = Column::with_children(
                self.programs
                    .iter()
                    .map(|(prog_nb, prog)| {
                        let style = if self.cur_prog.map_or(false, |cur_prog| cur_prog == *prog_nb)
                        {
                            ui::style::Button::ListItemSelected
                        } else {
                            ui::style::Button::ListItem
                        };

                        iced::widget::Button::new(row![
                            ui::value_label(prog_nb.to_string()),
                            horizontal_space(Length::Units(5)),
                            ui::value_label(prog.name().to_string()).width(Length::Fill),
                        ])
                        .on_press(SelectProgram(*prog_nb))
                        .style(style.into())
                        .into()
                    })
                    .collect(),
            );

            column![
                row![
                    ui::button("Utility Settings").on_press(ShowUtilitySettings),
                    horizontal_space(Length::Units(20)),
                    ui::button("Midi Connection").on_press(ShowMidiConnection),
                    horizontal_space(Length::Fill),
                    ui::checkbox("Dark Theme", self.use_dark_them, UseDarkTheme),
                ]
                .width(Length::Fill),
                vertical_space(Length::Units(10)),
                row![dspes, horizontal_space(Length::Units(10)), progs]
            ]
            .into()
        };

        let content: Element<_> = container(column![
            content,
            vertical_space(Length::Fill),
            text(&self.output_text).size(18),
        ])
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        // Set to true to debug layout
        if false {
            content.explain(iced::Color::WHITE)
        } else {
            content
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    JStation(Result<jstation::Message, jstation::Error>),
    Parameter(dsp::Parameter),
    UtilitySettings(dsp::UtilitySettings),
    Midi(ui::midi::Selection),
    SelectProgram(ProgramNumber),
    StartScan,
    ShowUtilitySettings,
    ShowMidiConnection,
    HideModal,
    UseDarkTheme(bool),
}

impl From<dsp::amp::Parameter> for Message {
    fn from(param: dsp::amp::Parameter) -> Self {
        Message::Parameter(param.into())
    }
}

impl From<dsp::cabinet::Parameter> for Message {
    fn from(param: dsp::cabinet::Parameter) -> Self {
        Message::Parameter(param.into())
    }
}

impl From<dsp::compressor::Parameter> for Message {
    fn from(param: dsp::compressor::Parameter) -> Self {
        Message::Parameter(param.into())
    }
}

impl From<dsp::delay::Parameter> for Message {
    fn from(param: dsp::delay::Parameter) -> Self {
        Message::Parameter(param.into())
    }
}

impl From<dsp::effect::Parameter> for Message {
    fn from(param: dsp::effect::Parameter) -> Self {
        Message::Parameter(param.into())
    }
}

impl From<dsp::noise_gate::Parameter> for Message {
    fn from(param: dsp::noise_gate::Parameter) -> Self {
        Message::Parameter(param.into())
    }
}

impl From<dsp::reverb::Parameter> for Message {
    fn from(param: dsp::reverb::Parameter) -> Self {
        Message::Parameter(param.into())
    }
}

impl From<ui::utility_settings::Event> for Message {
    fn from(evt: ui::utility_settings::Event) -> Self {
        use ui::utility_settings::Event::*;
        match evt {
            UtilitySettings(settings) => Message::UtilitySettings(settings),
            Parameter(param) => Message::Parameter(param.into()),
        }
    }
}

impl From<dsp::wah_expr::Parameter> for Message {
    fn from(param: dsp::wah_expr::Parameter) -> Self {
        Message::Parameter(param.into())
    }
}

#[derive(Debug, Copy, Clone, Hash)]
pub enum Subscription {
    JStation(usize),
}
