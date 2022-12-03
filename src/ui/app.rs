use iced::{
    widget::{button, checkbox, column, container, row, text, vertical_space},
    Application, Command, Element, Length, Theme,
};
use iced_native::command::Action;
use once_cell::sync::Lazy;
use smol::future::FutureExt;

use std::{cell::RefCell, future, rc::Rc, sync::Arc};

pub static APP_NAME: Lazy<Arc<str>> = Lazy::new(|| "J-Station Controller".into());

use crate::{jstation, midi, ui};

#[derive(Debug, Clone)]
pub enum Message {
    Ports(ui::port::Selection),
    StartScan,
    ShowUtilitySettings(bool),
    ShowMidiPanel(bool),
    JStation(Result<jstation::Message, jstation::Error>),
    UtilitySettings(ui::utility_settings::Event),
}

impl From<ui::utility_settings::Event> for Message {
    fn from(evt: ui::utility_settings::Event) -> Self {
        Message::UtilitySettings(evt)
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
    show_midi_panel: bool,
    ports: Rc<RefCell<ui::port::Ports>>,
    scanner_ctx: Option<midi::scanner::Context>,
    show_utility_settings: bool,
    utility_settings: jstation::procedure::UtilitySettingsResp,
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
                        self.utility_settings = *resp;

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
                log::info!("Unhandled {:?}", cv.msg);
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
            Err(err) => Err(err)?,
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
        let mut ports = ui::port::Ports::default();

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

            show_midi_panel: false,
            ports: RefCell::new(ports).into(),
            scanner_ctx: None,

            show_utility_settings: false,
            utility_settings: Default::default(),

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
        Theme::Dark
    }

    fn update(&mut self, event: Message) -> Command<Message> {
        use Message::*;
        match event {
            JStation(res) => {
                if let Err(err) = self.handle_jstation_event(res) {
                    self.show_error(&err);
                }
            }
            ShowMidiPanel(must_show) => self.show_midi_panel = must_show,
            Ports(ui::port::Selection { port_in, port_out }) => {
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
            UtilitySettings(event) => {
                use ui::utility_settings::Event::*;
                dbg!(&event);
                match event {
                    UtilitySettings(settings) => self.utility_settings = settings,
                    DigitalOutLevel(val) => {
                        self.utility_settings.digital_out_level = val;
                    }
                }
            }
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
                    ui::port::Panel::new(self.ports.clone(), Ports),
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
                    self.utility_settings,
                    UtilitySettings,
                ))
                .padding(5),
            );
        }

        let content = column![
            row![utility_settings, midi_panel]
                .spacing(20)
                .width(Length::Fill),
            vertical_space(Length::Fill),
            text(&self.output_text),
        ];

        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
