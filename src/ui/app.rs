use std::{cell::RefCell, future, rc::Rc, sync::Arc};

use iced::{
    widget::{button, checkbox, column, container, horizontal_space, row, text, vertical_space},
    Application, Command, Element, Length, Theme,
};
use iced_native::command::Action;
use once_cell::sync::Lazy;
use smol::future::FutureExt;

pub static APP_NAME: Lazy<Arc<str>> = Lazy::new(|| "J-Station Controller".into());

use crate::{
    jstation::{self, data::dsp},
    midi, ui,
};

#[derive(Debug, Clone)]
pub enum Message {
    JStation(Result<jstation::Message, jstation::Error>),
    Parameter(dsp::Parameter),
    UtilitySettings,
    Midi(ui::midi::Selection),
    StartScan,
    ShowUtilitySettings(bool),
    ShowMidiPanel(bool),
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

impl From<dsp::noise_gate::Parameter> for Message {
    fn from(param: dsp::noise_gate::Parameter) -> Self {
        Message::Parameter(param.into())
    }
}

impl From<ui::utility_settings::Event> for Message {
    fn from(evt: ui::utility_settings::Event) -> Self {
        use ui::utility_settings::Event::*;
        match evt {
            UtilitySettings => Message::UtilitySettings,
            Parameter(param) => Message::Parameter(param.into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash)]
pub enum Subscription {
    JStation(usize),
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Couldn't find J-Station")]
    JStationNotFound,
    #[error("J-Station error: {}", .0)]
    JStation(#[from] jstation::Error),
}

pub struct App {
    jstation: ui::jstation::Interface,
    amp: Rc<RefCell<dsp::Amp>>,
    cabinet: Rc<RefCell<dsp::Cabinet>>,
    compressor: Rc<RefCell<dsp::Compressor>>,
    noise_gate: Rc<RefCell<dsp::NoiseGate>>,

    show_midi_panel: bool,
    ports: Rc<RefCell<ui::midi::Ports>>,
    scanner_ctx: Option<midi::scanner::Context>,

    show_utility_settings: bool,
    utility_settings: Rc<RefCell<dsp::UtilitySettings>>,

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
    ) -> Result<(), Error> {
        use jstation::Message::*;
        match res {
            Ok(SysEx(sysex)) => {
                use jstation::Procedure::*;
                match &sysex.as_ref().proc {
                    WhoAmIResp(resp) => {
                        self.jstation.have_who_am_i_resp(resp).map_err(|err| {
                            self.jstation.clear();
                            self.ports.borrow_mut().set_disconnected();

                            err
                        })?;

                        self.output_text = "Found J-Station".to_string();

                        let (port_in, port_out) =
                            self.jstation.connected_ports().expect("Not connected");
                        self.ports.borrow_mut().set_ports(port_in, port_out);
                    }
                    UtilitySettingsResp(resp) => {
                        *self.utility_settings.borrow_mut() = resp.try_into()?;

                        // FIXME handle ui consequence of the error
                        self.jstation.bank_dump()?;
                    }
                    OneProgramResp(resp) => {
                        log::debug!("{:?}", resp.prog);
                    }
                    EndBankDumpResp(_) => {
                        log::debug!("EndBankDumpResp");
                    }
                    other => {
                        log::debug!("Unhandled {other:?}");
                    }
                }
            }
            Ok(ChannelVoice(cv)) => {
                use jstation::channel_voice::Message::*;
                use jstation::data::ParameterSetter;
                use jstation::Parameter::*;
                match cv.msg {
                    CC(Amp(param)) => {
                        log::info!("Got {param:?}");
                        let _ = self.amp.borrow_mut().set(param);
                    }
                    CC(Compressor(param)) => {
                        log::info!("Got {param:?}");
                        let _ = self.compressor.borrow_mut().set(param);
                    }
                    CC(Cabinet(param)) => {
                        log::info!("Got {param:?}");
                        let _ = self.cabinet.borrow_mut().set(param);
                    }
                    CC(NoiseGate(param)) => {
                        log::info!("Got {param:?}");
                        let _ = self.noise_gate.borrow_mut().set(param);
                    }
                    CC(UtilitySettings(util_settings)) => log::info!("Got {util_settings:?}"),
                    ProgramChange(pc) => {
                        log::info!("Unhandled {pc:?}");
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
                    self.show_midi_panel = true;

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

        Ok(())
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

            amp: Rc::new(RefCell::new(dsp::Amp::default())),
            cabinet: Rc::new(RefCell::new(dsp::Cabinet::default())),
            compressor: Rc::new(RefCell::new(dsp::Compressor::default())),
            noise_gate: Rc::new(RefCell::new(dsp::NoiseGate::default())),

            show_midi_panel: false,
            ports: RefCell::new(ports).into(),
            scanner_ctx: None,

            show_utility_settings: false,
            utility_settings: Default::default(),

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
            JStation(res) => {
                if let Err(err) = self.handle_jstation_event(res) {
                    self.show_error(&err);
                }
            }
            Parameter(param) => {
                use jstation::data::CCParameter;
                if let Some(cc) = param.to_cc() {
                    // FIXME handle the error
                    dbg!(&cc);
                    let _ = self.jstation.send_cc(cc);
                } else {
                    log::error!("No CC impl for {:?}", param);
                }
            }
            ShowMidiPanel(must_show) => self.show_midi_panel = must_show,
            Midi(ui::midi::Selection { port_in, port_out }) => {
                use midi::Scannable;
                if let Err(err) = self.jstation.connect(port_in, port_out) {
                    self.jstation.clear();
                    self.ports.borrow_mut().set_disconnected();
                    self.show_error(&err);
                }
            }
            StartScan => {
                log::debug!("Scanning Midi ports for J-Station");
                self.scanner_ctx = self.jstation.start_scan();

                if self.scanner_ctx.is_none() {
                    self.output_text = "Couldn't scan for J-Station".to_string();
                }
            }
            ShowUtilitySettings(must_show) => self.show_utility_settings = must_show,
            UtilitySettings => {
                log::debug!("Got UtilitySettings UI update");
                // FIXME send message to device
            }
            UseDarkTheme(use_dark) => self.use_dark_them = use_dark,
        }

        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        self.jstation.subscription().map(Message::JStation)
    }

    fn view(&self) -> Element<Message> {
        use Message::*;

        let mut midi_panel = column![checkbox(
            "Midi Connection",
            self.show_midi_panel,
            ShowMidiPanel
        )];
        if self.show_midi_panel {
            midi_panel = midi_panel.push(
                column![
                    ui::midi::Panel::new(self.ports.clone(), Midi),
                    button(text("Scan")).on_press(StartScan),
                ]
                .spacing(10)
                .padding(5),
            );
        }

        let mut utility_settings = column![checkbox(
            "Utility Settings",
            self.show_utility_settings,
            ShowUtilitySettings
        ),];
        if self.show_utility_settings {
            utility_settings = utility_settings.push(
                container(ui::utility_settings::Panel::new(
                    self.utility_settings.clone(),
                    Message::from,
                ))
                .padding(5),
            );
        }

        let content = column![
            row![
                utility_settings,
                midi_panel,
                horizontal_space(Length::Fill),
                checkbox("Dark Theme", self.use_dark_them, UseDarkTheme),
            ]
            .spacing(20)
            .width(Length::Fill),
            ui::compressor::Panel::new(self.compressor.clone()),
            ui::amp::Panel::new(self.amp.clone()),
            row![
                ui::cabinet::Panel::new(self.cabinet.clone()),
                ui::noise_gate::Panel::new(self.noise_gate.clone()),
            ]
            .spacing(30),
            vertical_space(Length::Fill),
            text(&self.output_text).size(super::LABEL_TEXT_SIZE),
        ]
        .spacing(40);

        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
