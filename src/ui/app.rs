use std::{borrow::Cow, cell::RefCell, future, rc::Rc, sync::Arc};

use iced::{
    widget::{column, container, horizontal_space, row, scrollable, vertical_space, Column, Text},
    Alignment, Application, Command, Element, Length, Theme,
};
use iced_native::command::Action;
use once_cell::sync::Lazy;
use smol::future::FutureExt;

use crate::jstation::{
    self,
    data::{dsp, Program, ProgramId, ProgramNb, ProgramsBank},
    prelude::*,
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
    status_text: Cow<'static, str>,
}

impl App {
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

                        self.set_status("Found J-Station");

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

    fn set_status(&mut self, status: impl Into<Cow<'static, str>>) {
        self.status_text = status.into();
    }

    fn clear_status(&mut self) {
        self.status_text = Default::default();
    }

    fn show_error(&mut self, err: impl ToString) {
        let err = err.to_string();
        log::error!("{err}");
        self.status_text = err.into();
    }

    fn refresh_ports(&mut self) {
        match self.jstation.refresh() {
            Ok(()) => self.ports.borrow_mut().update_from(self.jstation.iface()),
            Err(err) => self.show_error(format!("Midi ports not found: {err}")),
        }
    }
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        let mut app = App {
            jstation: ui::JStation::new(),

            ports: RefCell::new(ui::midi::Ports::default()).into(),
            scanner_ctx: None,

            panel: Panel::default(),
            use_dark_them: true,
            status_text: Default::default(),
        };

        app.refresh_ports();

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
        let res = match event {
            JStation(res) => match self.handle_device_evt(res) {
                Ok(cmd) => {
                    self.clear_status();
                    return cmd;
                }
                Err(err) => Err(err),
            },
            Parameter(param) => {
                self.jstation.update_param(param);
                Ok(())
            }
            SelectProgram(prog_id) => self.jstation.change_program(prog_id).map_err(Into::into),
            StoreTo(prog_nb) => {
                self.panel = Panel::Main;
                self.jstation.store_to(prog_nb).map_err(Into::into)
            }
            ShowStoreTo => {
                self.panel = Panel::StoreTo;
                Ok(())
            }
            Undo => self.jstation.undo().map_err(Into::into),
            Rename(name) => {
                self.jstation.rename(name);
                Ok(())
            }
            HideModal => {
                if self.panel.is_tuner() {
                    let _ = self.jstation.tuner_off();
                }

                self.panel = Panel::Main;
                Ok(())
            }
            SelectProgramsBank(bank) => {
                self.jstation.select_bank(bank);
                Ok(())
            }
            StartScan => {
                log::debug!("Scanning Midi ports for J-Station");
                self.scanner_ctx = self.jstation.start_scan();

                if self.scanner_ctx.is_none() {
                    self.set_status("Couldn't scan for J-Station");
                }

                return Command::none();
            }
            UtilitySettings(settings) => {
                self.jstation.update_utility_settings(settings);
                Ok(())
            }
            Midi(ui::midi::Selection { port_in, port_out }) => {
                use midi::Scannable;
                self.jstation
                    .connect(port_in, port_out)
                    .map(|_| ())
                    .map_err(|err| {
                        self.jstation.clear();
                        self.ports.borrow_mut().set_disconnected();
                        err.into()
                    })
            }
            UseDarkTheme(use_dark) => {
                self.use_dark_them = use_dark;
                Ok(())
            }
            ShowMidiConnection => {
                self.panel = Panel::MidiConnection;
                Ok(())
            }
            ShowTuner => match self.jstation.tuner_on() {
                Ok(()) => {
                    self.panel = Panel::Tuner;
                    Ok(())
                }
                Err(err) => Err(err.into()),
            },
            ShowUtilitySettings => {
                self.panel = Panel::UtilitySettings;
                Ok(())
            }
        };

        match res {
            Ok(()) => self.clear_status(),
            Err(err) => self.show_error(err),
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
                let mut dsp = column![
                    ui::compressor::Panel::new(self.jstation.dsp().compressor),
                    ui::wah_expr::Panel::new(
                        self.jstation.dsp().expression,
                        self.jstation.dsp().pedal,
                        self.jstation.dsp().wah,
                    ),
                ]
                .spacing(11);

                let effect_post = self.jstation.dsp().effect.post;

                if !effect_post.is_true() {
                    dsp = dsp.push(ui::effect::Panel::new(self.jstation.dsp().effect));
                }

                dsp = dsp.push(ui::amp::Panel::new(self.jstation.dsp().amp));

                dsp = dsp.push(row![
                    ui::dsp_keep_width(ui::cabinet::Panel::new(self.jstation.dsp().cabinet)),
                    horizontal_space(Length::Units(10)),
                    ui::dsp_keep_width(ui::noise_gate::Panel::new(self.jstation.dsp().noise_gate)),
                ]);

                if effect_post.is_true() {
                    dsp = dsp.push(ui::effect::Panel::new(self.jstation.dsp().effect));
                }

                dsp = dsp.push(ui::delay::Panel::new(self.jstation.dsp().delay));
                dsp = dsp.push(ui::reverb::Panel::new(self.jstation.dsp().reverb));

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
                    ui::button("Settings...")
                        .on_press(ShowUtilitySettings)
                        .style(style::Button::Default.into()),
                    horizontal_space(Length::Units(10)),
                    ui::button("MIDI...")
                        .on_press(ShowMidiConnection)
                        .style(style::Button::Default.into()),
                    horizontal_space(Length::Units(10)),
                    ui::button("Tuner...")
                        .on_press(ShowTuner)
                        .style(style::Button::Default.into()),
                    horizontal_space(Length::Units(20)),
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
                            .style(style::Button::Active.into()),
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
                        scrollable(dsp),
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
            Panel::Tuner => ui::modal(
                "Tuner On",
                ui::button("Done")
                    .on_press(HideModal)
                    .style(style::Button::Active.into()),
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
                Text::new(self.status_text.clone())
                    .size(18)
                    .width(Length::Fill),
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
    ShowTuner,
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

impl From<dsp::Parameter> for Message {
    fn from(param: dsp::Parameter) -> Self {
        Message::Parameter(param)
    }
}

#[derive(Clone, Copy, Debug, Default)]
enum Panel {
    #[default]
    Main,
    StoreTo,
    MidiConnection,
    Tuner,
    UtilitySettings,
}

impl Panel {
    fn is_tuner(self) -> bool {
        matches!(self, Panel::Tuner)
    }
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
