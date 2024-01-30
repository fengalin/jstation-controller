use std::{borrow::Cow, cell::RefCell, rc::Rc, sync::Arc};

use iced::{
    widget::{column, row, text},
    Alignment, Element, Length,
};
use iced_lazy::{self, Component};

use once_cell::sync::Lazy;

use crate::jstation;
use crate::midi::{self, port::Direction};
use crate::ui;

static DISCONNECTED: Lazy<Arc<str>> = Lazy::new(|| "Disconnected".into());

#[derive(Debug)]
pub struct DirectionalPorts {
    pub list: Cow<'static, [Arc<str>]>,
    pub cur: Arc<str>,
}

impl DirectionalPorts {
    fn update_from<IO: midir::MidiIO>(&mut self, ports: &midi::DirectionalPorts<IO>) {
        self.list = Cow::from_iter(ports.list());
        self.cur = ports.cur().unwrap_or_else(|| DISCONNECTED.clone());
    }
}

impl Default for DirectionalPorts {
    fn default() -> Self {
        Self {
            list: vec![].into(),
            cur: DISCONNECTED.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Select((Direction, Arc<str>)),
}

#[derive(Debug, Clone)]
pub struct Selection {
    pub port_in: Arc<str>,
    pub port_out: Arc<str>,
}

impl From<(Direction, Arc<str>)> for Event {
    fn from(direction_port: (Direction, Arc<str>)) -> Self {
        Event::Select(direction_port)
    }
}

#[derive(Debug, Default)]
pub struct Ports {
    ins: DirectionalPorts,
    outs: DirectionalPorts,
}

impl Ports {
    pub fn update_from(&mut self, jstation: &jstation::Interface) {
        self.ins.update_from(&jstation.ins);
        self.outs.update_from(&jstation.outs);
    }

    pub fn set_ports(&mut self, port_in: Arc<str>, port_out: Arc<str>) {
        self.ins.cur = port_in;
        self.outs.cur = port_out;
    }

    pub fn set_disconnected(&mut self) {
        self.ins.cur = DISCONNECTED.clone();
        self.outs.cur = DISCONNECTED.clone();
    }
}

pub struct Panel<'a, Message> {
    ports: Rc<RefCell<Ports>>,
    on_change: Box<dyn 'a + Fn(Selection) -> Message>,
}

impl<'a, Message> Panel<'a, Message> {
    pub fn new<F>(ports: Rc<RefCell<Ports>>, on_change: F) -> Self
    where
        F: 'a + Fn(Selection) -> Message,
    {
        Self {
            ports,
            on_change: Box::new(on_change),
        }
    }
}

impl<'a, Message> Component<Message, iced::Renderer> for Panel<'a, Message> {
    type State = ();
    type Event = Event;

    fn update(&mut self, _state: &mut Self::State, event: Event) -> Option<Message> {
        use Direction::*;
        use Event::*;
        let mut ports = self.ports.borrow_mut();
        match event {
            Select((In, port_name)) => {
                ports.ins.cur = port_name;
            }
            Select((Out, port_name)) => {
                ports.outs.cur = port_name;
            }
        }

        if ports.ins.cur != *DISCONNECTED && ports.outs.cur != *DISCONNECTED {
            return Some((self.on_change)(Selection {
                port_in: ports.ins.cur.clone(),
                port_out: ports.outs.cur.clone(),
            }));
        }

        None
    }

    fn view(&self, _state: &Self::State) -> Element<Event, iced::Renderer> {
        use Direction::*;

        let ports = self.ports.borrow();
        let in_pick_list = ui::pick_list(
            ports.ins.list.clone(),
            Some(ports.ins.cur.clone()),
            |port| (In, port).into(),
        )
        .width(Length::Fill);
        let out_pick_list = ui::pick_list(
            ports.outs.list.clone(),
            Some(ports.outs.cur.clone()),
            |port| (Out, port).into(),
        )
        .width(Length::Fill);

        // This is to force the labels to occupy the same column
        // whatever the length of the labels. We would need
        // a grid layout here.
        let label_width = Length::Fixed(40f32);
        let in_label = text("In:").width(label_width).size(18);
        let out_label = text("Out:").width(label_width).size(18);

        let content: Element<_> = column![
            row![in_label, in_pick_list].align_items(Alignment::Center),
            row![out_label, out_pick_list].align_items(Alignment::Center)
        ]
        .spacing(10)
        .into();

        // Set to true to debug layout
        if false {
            content.explain(iced::Color::WHITE)
        } else {
            content
        }
    }
}

impl<'a, Message: 'a> From<Panel<'a, Message>> for Element<'a, Message, iced::Renderer> {
    fn from(panel: Panel<'a, Message>) -> Self {
        iced_lazy::component(panel)
    }
}
