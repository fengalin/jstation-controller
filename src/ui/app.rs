use std::{borrow::Cow, cell::RefCell, future, rc::Rc, sync::Arc};

use iced::{
    widget::{column, container, horizontal_space, row, scrollable, text, vertical_space, Column},
    Alignment, Application, Command, Element, Length, Theme,
};
use iced_native::command::Action;
use once_cell::sync::Lazy;
use smol::future::FutureExt;

use crate::jstation::{
    self,
    data::{dsp, Program, ProgramId, ProgramNb, ProgramsBank},
};
use crate::midi;
use crate::ui::{self, style, widget};

pub static APP_NAME: Lazy<Arc<str>> = Lazy::new(|| "J-Station Controller".into());

static PROGRAMS_BANKS: Lazy<Cow<'static, [ProgramsBank]>> =
    Lazy::new(|| vec![ProgramsBank::User, ProgramsBank::Factory].into());

pub struct App {
    jstation: ui::JStation,

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

    fn handle_device_evt(
        &mut self,
        res: Result<jstation::Message, jstation::Error>,
    ) -> Result<Command<Message>, Error> {
        use jstation::Message::*;
        match res {
            Ok(SysEx(sysex)) => {
                use jstation::Procedure::*;
                match sysex.proc {
                    WhoAmIResp(_) => {
                        self.jstation.handle_device(SysEx(sysex)).map_err(|err| {
                            self.ports.borrow_mut().set_disconnected();

                            err
                        })?;

                        self.output_text = "Found J-Station".to_string();

                        let (port_in, port_out) = self
                            .jstation
                            .iface()
                            .connected_ports()
                            .expect("Not connected");
                        self.ports.borrow_mut().set_ports(port_in, port_out);

                        return Ok(Command::single(Action::Future(
                            future::ready(Message::HideModal).boxed(),
                        )));
                    }
                    _ => self.jstation.handle_device(SysEx(sysex))?,
                }
            }
            Ok(ChannelVoice(cv)) => self.jstation.handle_device(ChannelVoice(cv))?,
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

        let mut jstation = ui::JStation::new();
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
            JStation(res) => match self.handle_device_evt(res) {
                Ok(cmd) => return cmd,
                Err(err) => self.show_error(&err),
            },
            Parameter(param) => self.jstation.update_param(param),
            SelectProgram(prog_id) => {
                if let Err(err) = self.jstation.change_program(prog_id) {
                    self.show_error(&err);
                }
            }
            StoreTo(prog_nb) => {
                self.panel = Panel::Main;

                if let Err(err) = self.jstation.store_to(prog_nb) {
                    self.show_error(&err);
                }
            }
            ShowStoreTo => {
                self.panel = Panel::StoreTo;
            }
            Undo => {
                if let Err(err) = self.jstation.undo() {
                    self.show_error(&err);
                }
            }
            Rename(name) => self.jstation.rename(name),
            HideModal => self.panel = Panel::Main,
            SelectProgramsBank(bank) => self.jstation.select_bank(bank),
            StartScan => {
                log::debug!("Scanning Midi ports for J-Station");
                self.scanner_ctx = self.jstation.start_scan();

                if self.scanner_ctx.is_none() {
                    self.output_text = "Couldn't scan for J-Station".to_string();
                }
            }
            UtilitySettings(settings) => {
                log::debug!("Got UtilitySettings UI update");
                self.jstation.update_utility_settings(settings);
            }
            Midi(ui::midi::Selection { port_in, port_out }) => {
                use midi::Scannable;
                if let Err(err) = self.jstation.connect(port_in, port_out) {
                    self.jstation.clear();
                    self.ports.borrow_mut().set_disconnected();
                    self.show_error(&err);
                }
            }
            UseDarkTheme(use_dark) => self.use_dark_them = use_dark,
            ShowMidiConnection => self.panel = Panel::MidiConnection,
            ShowUtilitySettings => self.panel = Panel::UtilitySettings,
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
                    ui::compressor::Panel::new(self.jstation.dsp().compressor),
                    ui::wah_expr::Panel::new(self.jstation.dsp().wah_expr),
                ]
                .spacing(11);

                let effect_post = self.jstation.dsp().effect.post;

                if !effect_post.is_true() {
                    dspes = dspes.push(ui::effect::Panel::new(self.jstation.dsp().effect));
                }

                dspes = dspes.push(ui::amp::Panel::new(self.jstation.dsp().amp));

                dspes = dspes.push(row![
                    ui::dsp_keep_width(ui::cabinet::Panel::new(self.jstation.dsp().cabinet)),
                    horizontal_space(Length::Units(10)),
                    ui::dsp_keep_width(ui::noise_gate::Panel::new(self.jstation.dsp().noise_gate)),
                ]);

                if effect_post.is_true() {
                    dspes = dspes.push(ui::effect::Panel::new(self.jstation.dsp().effect));
                }

                dspes = dspes.push(ui::delay::Panel::new(self.jstation.dsp().delay));
                dspes = dspes.push(ui::reverb::Panel::new(self.jstation.dsp().reverb));

                let progs = Column::with_children(
                    ProgramNb::enumerate()
                        .map(|prog_nb| {
                            let prog_id = ProgramId::new(self.jstation.programs_bank(), prog_nb);

                            let style = if self
                                .jstation
                                .cur_prog_id()
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
                                    self.jstation
                                        .get_program(prog_id)
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
                    horizontal_space(Length::Units(10)),
                    ui::button("MIDI...")
                        .on_press(ShowMidiConnection)
                        .style(style::Button::Default.into()),
                    horizontal_space(Length::Units(50)),
                    ui::text_input("program name", self.jstation.dsp().name.as_str(), Rename)
                        .width(Length::Units(200)),
                    horizontal_space(Length::Fill),
                ]
                .width(widget::DEFAULT_DSP_WIDTH);

                if self.jstation.iface().is_connected() {
                    if self.jstation.has_changed() {
                        left_header = left_header.push(
                            ui::button("Undo")
                                .on_press(Undo)
                                .style(style::Button::Default.into()),
                        );
                        left_header = left_header.push(horizontal_space(Length::Units(10)));
                    }

                    left_header = left_header.push(
                        ui::button("Store...")
                            .on_press(ShowStoreTo)
                            .style(style::Button::Store.into()),
                    );
                }

                let right_header = row![ui::pick_list(
                    PROGRAMS_BANKS.clone(),
                    Some(self.jstation.programs_bank()),
                    move |bank| { SelectProgramsBank(bank) }
                )
                .width(Length::Fill),];

                column![
                    row![
                        left_header,
                        horizontal_space(widget::DSP_PROGRAM_SPACING),
                        right_header
                    ],
                    vertical_space(Length::Units(10)),
                    row![
                        scrollable(dspes),
                        horizontal_space(widget::DSP_PROGRAM_SPACING),
                        scrollable(progs),
                    ],
                ]
                .into()
            }
            Panel::StoreTo => {
                let progs = scrollable(Column::with_children(
                    ProgramNb::enumerate()
                        .map(|prog_nb| {
                            let prog_id = ProgramId::new_user(prog_nb);

                            let style = if self
                                .jstation
                                .cur_prog_id()
                                .map_or(false, |cur_prog_id| cur_prog_id.nb() == prog_id.nb())
                            {
                                ui::style::Button::ListItemSelected
                            } else {
                                ui::style::Button::ListItem
                            };

                            iced::widget::Button::new(row![
                                ui::value_label(prog_id.nb().to_string()),
                                horizontal_space(Length::Units(5)),
                                ui::value_label(
                                    self.jstation
                                        .get_program(prog_id)
                                        .map_or("", Program::name)
                                        .to_string()
                                )
                                .width(Length::Fill),
                            ])
                            .on_press(StoreTo(prog_id.nb()))
                            .style(style.into())
                            .into()
                        })
                        .collect(),
                ));

                ui::modal("Store to...", progs, HideModal).into()
            }
            Panel::MidiConnection => ui::modal(
                "MIDI Connection",
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
                "Utility Settings",
                ui::utility_settings::Panel::new(
                    self.jstation.dsp().utility_settings,
                    Message::from,
                ),
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
                ui::checkbox(self.use_dark_them, "Dark Theme", UseDarkTheme),
            ],
        ])
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::Background)
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
    Midi(ui::midi::Selection),
    Rename(String),
    SelectProgram(ProgramId),
    SelectProgramsBank(ProgramsBank),
    ShowUtilitySettings,
    ShowMidiConnection,
    ShowStoreTo,
    StartScan,
    StoreTo(ProgramNb),
    Undo,
    HideModal,
    UseDarkTheme(bool),
    UtilitySettings(dsp::UtilitySettings),
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
    StoreTo,
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
}

#[derive(Debug, Copy, Clone, Hash)]
pub enum Subscription {
    JStation(usize),
}
