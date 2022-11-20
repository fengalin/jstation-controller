use iced::{
    widget::{column, container, row, text},
    Alignment, Application, Command, Element, Length, Subscription, Theme,
};
use iced_audio::{
    text_marks, tick_marks, FreqRange, Knob, LogDBRange, Normal, NormalParam, VSlider,
};
use once_cell::sync::Lazy;

use std::{cell::RefCell, rc::Rc, sync::Arc};

pub static APP_NAME: Lazy<Arc<str>> = Lazy::new(|| "J-Station Controller".into());

use crate::{jstation, ui};

// The message when a parameter widget is moved by the user
#[derive(Debug, Clone)]
pub enum Message {
    VSliderDB(Normal),
    KnobFreq(Normal),
    Ports(ui::port::Selection),
    JStation(Result<jstation::Message, jstation::Error>),
}

pub struct App {
    jstation: jstation::Interface,
    ports: Rc<RefCell<ui::port::Ports>>,

    db_range: LogDBRange,
    freq_range: FreqRange,

    db_param: NormalParam,
    freq_param: NormalParam,

    db_tick_marks: tick_marks::Group,
    db_text_marks: text_marks::Group,

    freq_tick_marks: tick_marks::Group,
    freq_text_marks: text_marks::Group,

    output_text: String,
}

impl App {
    fn handle_error(&mut self, err: &dyn std::error::Error) {
        log::error!("{err}");
        self.output_text = err.to_string();
        // FIXME probably need to refresh UI
    }
}

impl Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (App, Command<Message>) {
        let mut output_text = " ".to_string();

        let mut jstation = jstation::Interface::new(APP_NAME.clone());
        let mut ports = ui::port::Ports::default();

        match jstation.refresh() {
            Ok(()) => ports.update_from(&jstation.ins, &jstation.outs),
            Err(err) => {
                // FIXME set a flag to indicate the application can't be used as is
                let msg = format!("Midi ports not found: {err}");
                log::error!("{msg}");
                output_text = msg;
            }
        }

        let db_range = LogDBRange::new(-60.0, 6.0, 0.8.into());
        let freq_range = FreqRange::default();

        let app = App {
            jstation,
            ports: RefCell::new(ports).into(),

            db_range,
            freq_range,

            db_param: db_range.default_normal_param(),
            freq_param: freq_range.normal_param(1000.0, 1000.0),

            db_tick_marks: vec![
                (db_range.map_to_normal(6.0), tick_marks::Tier::Two),
                (db_range.map_to_normal(5.0), tick_marks::Tier::Three),
                (db_range.map_to_normal(4.0), tick_marks::Tier::Three),
                (db_range.map_to_normal(3.0), tick_marks::Tier::Two),
                (db_range.map_to_normal(2.0), tick_marks::Tier::Three),
                (db_range.map_to_normal(1.0), tick_marks::Tier::Three),
                (db_range.map_to_normal(0.0), tick_marks::Tier::One),
                (db_range.map_to_normal(-1.0), tick_marks::Tier::Three),
                (db_range.map_to_normal(-2.0), tick_marks::Tier::Three),
                (db_range.map_to_normal(-3.0), tick_marks::Tier::Two),
                (db_range.map_to_normal(-5.0), tick_marks::Tier::Three),
                (db_range.map_to_normal(-6.0), tick_marks::Tier::Two),
                (db_range.map_to_normal(-7.0), tick_marks::Tier::Three),
                (db_range.map_to_normal(-8.0), tick_marks::Tier::Three),
                (db_range.map_to_normal(-9.0), tick_marks::Tier::Two),
                (db_range.map_to_normal(-10.0), tick_marks::Tier::Two),
                (db_range.map_to_normal(-20.0), tick_marks::Tier::Two),
                (db_range.map_to_normal(-40.0), tick_marks::Tier::Two),
            ]
            .into(),

            db_text_marks: vec![
                (db_range.map_to_normal(6.0), "+6"),
                (db_range.map_to_normal(3.0), "+3"),
                (db_range.map_to_normal(0.0), "0"),
                (db_range.map_to_normal(-3.0), "-3"),
                (db_range.map_to_normal(-6.0), "-6"),
                (db_range.map_to_normal(-10.0), "-10"),
                (db_range.map_to_normal(-20.0), "-20"),
                (db_range.map_to_normal(-40.0), "-40"),
            ]
            .into(),

            freq_tick_marks: vec![
                (freq_range.map_to_normal(20.0), tick_marks::Tier::Two),
                (freq_range.map_to_normal(50.0), tick_marks::Tier::Two),
                (freq_range.map_to_normal(100.0), tick_marks::Tier::One),
                (freq_range.map_to_normal(200.0), tick_marks::Tier::Two),
                (freq_range.map_to_normal(400.0), tick_marks::Tier::Two),
                (freq_range.map_to_normal(1000.0), tick_marks::Tier::One),
                (freq_range.map_to_normal(2000.0), tick_marks::Tier::Two),
                (freq_range.map_to_normal(5000.0), tick_marks::Tier::Two),
                (freq_range.map_to_normal(10000.0), tick_marks::Tier::One),
                (freq_range.map_to_normal(20000.0), tick_marks::Tier::Two),
            ]
            .into(),

            freq_text_marks: vec![
                (freq_range.map_to_normal(100.0), "100"),
                (freq_range.map_to_normal(1000.0), "1k"),
                (freq_range.map_to_normal(10000.0), "10k"),
            ]
            .into(),

            output_text,
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        APP_NAME.to_string()
    }

    fn update(&mut self, event: Message) -> Command<Message> {
        use Message::*;
        match event {
            VSliderDB(normal) => {
                self.db_param.update(normal);

                let value = self.db_range.unmap_to_value(normal);
                self.output_text = format!("VSliderDB: {:.3}", value);
            }
            KnobFreq(normal) => {
                self.freq_param.update(normal);

                let value = self.freq_range.unmap_to_value(normal);
                self.output_text = format!("KnobFreq: {:.2}", value);
            }
            Ports(ui::port::Selection { port_in, port_out }) => {
                if let Err(err) = self.jstation.connect(port_in, port_out) {
                    self.handle_error(err.as_ref());
                }
            }
            JStation(res) => match res {
                // FIXME move this in a dedicate function
                Ok(jstation::Message::SysEx(sysex)) => {
                    use jstation::Procedure::*;
                    match &sysex.as_ref().proc {
                        WhoAmIResp(resp) => {
                            if let Err(err) = self.jstation.have_who_am_i_resp(resp) {
                                self.handle_error(&err);
                            } else {
                                self.output_text = "Found J-Station".to_string();
                            }
                        }
                        other => {
                            log::debug!("Unhandled {other:?}");
                        }
                    }
                }
                Ok(jstation::Message::ChannelVoice(cv)) => {
                    log::debug!("Unhandled {:?}", cv.msg);
                }
                Err(err) => {
                    self.handle_error(&err);
                }
            },
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        self.jstation.subscription().map(Message::JStation)
    }

    fn view(&self) -> Element<Message> {
        //let scan_btn = button(text("Scan")).on_press(Message::Scan);
        let content = row![
            column![
                Knob::new(self.freq_param, Message::KnobFreq, || None, || None)
                    .size(Length::Units(50))
                    .tick_marks(&self.freq_tick_marks)
                    .text_marks(&self.freq_text_marks),
                ui::port::Panel::new(self.ports.clone(), Message::Ports),
                text(&self.output_text).width(Length::Fill),
                //scan_btn,
            ]
            .max_width(900)
            .spacing(20)
            .padding(20)
            .align_items(Alignment::Center),
            container(
                VSlider::new(self.db_param, Message::VSliderDB)
                    .tick_marks(&self.db_tick_marks)
                    .text_marks(&self.db_text_marks)
            )
            .height(Length::Fill)
            .max_height(300),
        ]
        .spacing(40);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
