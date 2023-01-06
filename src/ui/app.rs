use std::{borrow::Cow, cell::RefCell, collections::BTreeMap, future, rc::Rc, sync::Arc};

use iced::{
    widget::{column, container, horizontal_space, row, text, vertical_space, Column},
    Alignment, Application, Command, Element, Length, Theme,
};
use iced_native::command::Action;
use once_cell::sync::Lazy;
use smol::future::FutureExt;

use crate::jstation::{
    self,
    data::{dsp, BaseParameter, Program, ProgramId, ProgramNb, ProgramsBank, RawParameterSetter},
};
use crate::midi;
use crate::ui::{self, style, widget};

pub static APP_NAME: Lazy<Arc<str>> = Lazy::new(|| "J-Station Controller".into());

static PROGRAMS_BANKS: Lazy<Cow<'static, [ProgramsBank]>> =
    Lazy::new(|| vec![ProgramsBank::User, ProgramsBank::Factory].into());

pub struct App {
    jstation: ui::jstation::Interface,
    dsp: dsp::Dsp,
    progs_bank: ProgramsBank,
    programs: BTreeMap<ProgramId, Program>,
    cur_prog_id: Option<ProgramId>,
    has_changed: bool,

    ports: Rc<RefCell<ui::midi::Ports>>,
    scanner_ctx: Option<midi::scanner::Context>,

    panel: Panel,
    use_dark_them: bool,
    output_text: String,
}

impl App {
    fn show_error(&mut self, err: impl ToString) {
        let err = err.to_string();
        log::error!("{err}");
        self.output_text = err;
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
                    ProgramIndicesResp(_) => (),
                    OneProgramResp(resp) => {
                        if self.cur_prog_id.is_some() {
                            self.dsp.set_raw(resp.prog.data())?;
                        }

                        self.programs.insert(resp.prog.id(), resp.prog);
                    }
                    ProgramUpdateResp(resp) => {
                        let prog = self
                            .programs
                            .iter()
                            .find(|(_, prog)| *prog == &resp.prog_data);

                        if let Some((_, prog)) = prog {
                            self.cur_prog_id = Some(prog.id());
                            log::debug!(
                                "Program Update {}. Has-changed flag: {}",
                                prog.id(),
                                resp.has_changed
                            );
                        } else {
                            // This can occur on startup when the program on device `has_changed`
                            // or if a factory program is selected.
                            self.cur_prog_id = None;
                            self.show_error(&Error::ProgramIdenticationFailure);
                        }

                        self.dsp.set_raw(resp.prog_data.data())?;
                        self.has_changed = resp.has_changed;
                    }
                    StartBankDumpResp(_) => {
                        self.progs_bank = ProgramsBank::default();
                    }
                    EndBankDumpResp(_) => {
                        self.jstation.program_update_req()?;
                    }
                    ToMessageResp(resp) => match resp.res {
                        Ok(req_proc) => log::debug!("Proc. x{req_proc:02x}: Ok"),
                        Err(err) => panic!("{err}"),
                    },
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
                        Ok(Some(param)) => {
                            let cur_prog =
                                self.cur_prog_id.map(|prog_id| self.programs.get(&prog_id));
                            if let Some(cur_prog) = cur_prog {
                                // FIXME
                                /*
                                if param.raw_value() != cur_prog.data[param.] {
                                }
                                */
                            }
                        }
                        Ok(None) => log::debug!("Unchanged param for {cc:?}"),
                        Err(err) => log::warn!("{err}"),
                    },
                    ProgramChange(prog_id) => {
                        self.cur_prog_id = Some(prog_id);
                        self.progs_bank = prog_id.progs_bank();

                        if let Some(prog) = self.programs.get(&prog_id) {
                            self.dsp.set_raw(prog.data()).unwrap();
                        } else if let Err(err) = self.jstation.request_program(prog_id) {
                            self.cur_prog_id = None;
                            self.show_error(&err);
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
                    self.panel = Panel::MidiConnection;

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
            progs_bank: ProgramsBank::default(),
            programs: BTreeMap::new(),
            cur_prog_id: None,
            has_changed: false,

            ports: RefCell::new(ports).into(),
            scanner_ctx: None,

            panel: Panel::default(),
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

                self.dsp.set(param).unwrap();

                if let Some(cc) = param.to_cc() {
                    // FIXME handle the error
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
            SelectProgram(prog_id) => match self.jstation.change_program(prog_id) {
                Ok(()) => {
                    self.cur_prog_id = Some(prog_id);
                    self.has_changed = false;

                    if let Some(prog) = self.programs.get(&prog_id) {
                        self.dsp.set_raw(prog.data()).unwrap();
                    } else if let Err(err) = self.jstation.request_program(prog_id) {
                        self.cur_prog_id = None;
                        self.show_error(&err);
                    }
                }
                Err(err) => self.show_error(&err),
            },
            SelectProgramsBank(progs_bank) => {
                self.progs_bank = progs_bank;
            }
            Store => {
                // TODO
            }
            StoreTo => {
                // TODO
            }
            Undo => {
                if let Err(err) = self.jstation.reload_program() {
                    self.show_error(&err);
                }

                if let Some(cur_prog_id) = self.cur_prog_id {
                    if let Some(prog) = self.programs.get(&cur_prog_id) {
                        self.dsp.set_raw(prog.data()).unwrap();
                    } else if let Err(err) = self.jstation.request_program(cur_prog_id) {
                        self.cur_prog_id = None;
                        self.show_error(&err);
                    }
                }

                self.has_changed = false;
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
            ShowMidiConnection => self.panel = Panel::MidiConnection,
            ShowUtilitySettings => self.panel = Panel::UtilitySettings,
            HideModal => self.panel = Panel::Main,
        }

        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        self.jstation.subscription().map(Message::JStation)
    }

    fn view(&self) -> Element<Message> {
        use Message::*;

        let content: Element<_> = match self.panel {
            Panel::Main => {
                use jstation::data::BoolParameter;

                let mut dspes = column![
                    ui::compressor::Panel::new(self.dsp.compressor),
                    ui::wah_expr::Panel::new(self.dsp.wah_expr),
                ]
                .spacing(11);

                let effect_post = self.dsp.effect.post;

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
                    ProgramNb::enumerate()
                        .map(|prog_nb| {
                            let prog_id = ProgramId::new(self.progs_bank, prog_nb);

                            let style = if self
                                .cur_prog_id
                                .map_or(false, |cur_prog_id| cur_prog_id == prog_id)
                            {
                                ui::style::Button::ListItemSelected
                            } else {
                                ui::style::Button::ListItem
                            };

                            iced::widget::Button::new(row![
                                ui::value_label(prog_id.nb().to_string()),
                                horizontal_space(Length::Units(5)),
                                ui::value_label(
                                    self.programs
                                        .get(&prog_id)
                                        .map_or("", Program::name)
                                        .to_string()
                                )
                                .width(Length::Fill),
                            ])
                            .on_press(SelectProgram(prog_id))
                            .style(style.into())
                            .into()
                        })
                        .collect(),
                );

                let mut left_header = row![
                    ui::button("Utility Settings...")
                        .on_press(ShowUtilitySettings)
                        .style(style::Button::Default.into()),
                    horizontal_space(Length::Units(20)),
                    ui::button("MIDI Connection...")
                        .on_press(ShowMidiConnection)
                        .style(style::Button::Default.into()),
                ]
                .width(widget::DEFAULT_DSP_WIDTH);

                if self.has_changed {
                    left_header = left_header.push(horizontal_space(Length::Fill));

                    if self
                        .cur_prog_id
                        .map_or(false, |cur_prog_id| cur_prog_id.progs_bank().is_user())
                    {
                        left_header = left_header.push(
                            ui::button("Store")
                                .on_press(Store)
                                .style(style::Button::Store.into()),
                        );
                        left_header = left_header.push(horizontal_space(Length::Units(10)));
                    }

                    left_header = left_header.push(
                        ui::button("Store to...")
                            .on_press(StoreTo)
                            .style(style::Button::Store.into()),
                    );
                    left_header = left_header.push(horizontal_space(Length::Units(20)));
                    left_header = left_header.push(
                        ui::button("Undo")
                            .on_press(Undo)
                            .style(style::Button::Default.into()),
                    );
                }

                let right_header = row![
                    ui::pick_list(
                        PROGRAMS_BANKS.clone(),
                        Some(self.progs_bank),
                        move |progs_bank| { SelectProgramsBank(progs_bank) }
                    )
                    .width(Length::Units(100)),
                    // FIXME add rename button
                ];

                column![
                    row![
                        left_header,
                        horizontal_space(widget::DSP_PROGRAM_SPACING),
                        right_header
                    ],
                    vertical_space(Length::Units(10)),
                    row![dspes, horizontal_space(widget::DSP_PROGRAM_SPACING), progs]
                ]
                .into()
            }
            Panel::MidiConnection => ui::modal(
                column![
                    ui::midi::Panel::new(self.ports.clone(), Midi),
                    vertical_space(Length::Units(20)),
                    ui::button("Scan")
                        .on_press(StartScan)
                        .style(style::Button::Default.into()),
                ]
                .align_items(Alignment::End),
                HideModal,
            )
            .into(),
            Panel::UtilitySettings => ui::modal(
                ui::utility_settings::Panel::new(self.dsp.utility_settings, Message::from),
                HideModal,
            )
            .into(),
        };

        let content: Element<_> = container(column![
            content,
            vertical_space(Length::Fill),
            row![
                text(&self.output_text).size(18),
                horizontal_space(Length::Fill),
                ui::checkbox("Dark Theme", self.use_dark_them, UseDarkTheme),
            ],
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
    SelectProgram(ProgramId),
    SelectProgramsBank(ProgramsBank),
    StartScan,
    ShowUtilitySettings,
    ShowMidiConnection,
    Store,
    StoreTo,
    Undo,
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

#[derive(Debug, Default)]
enum Panel {
    #[default]
    Main,
    MidiConnection,
    UtilitySettings,
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Couldn't find J-Station")]
    JStationNotFound,
    #[error("J-Station error: {}", .0)]
    JStation(#[from] jstation::Error),
    #[error("Unexpected program received {}, expected {}", .received, .expected)]
    UnexpectedProgramNumber {
        received: ProgramId,
        expected: ProgramId,
    },
    #[error("Unexpected program received {}", .0)]
    UnexpectedProgram(ProgramId),
    #[error("Failed to identify Program update")]
    ProgramIdenticationFailure,
}

#[derive(Debug, Copy, Clone, Hash)]
pub enum Subscription {
    JStation(usize),
}
