use iced::{
    overlay,
    widget::{self, column, pick_list, row, text},
    Alignment, Element, Length,
};
use iced_lazy::{self, Component};

use once_cell::sync::Lazy;

use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{
    jstation,
    midi::{self, port::Direction},
};

static DISCONNECTED: Lazy<Arc<str>> = Lazy::new(|| "Disconnected".into());

#[derive(Debug)]
pub struct DirectionalPorts {
    pub list: Vec<Arc<str>>,
    pub cur: Arc<str>,
}

impl DirectionalPorts {
    fn update_from<IO: midir::MidiIO>(&mut self, ports: &midi::DirectionalPorts<IO>) {
        self.list.clear();
        self.list.extend(ports.list());

        self.cur = ports.cur().unwrap_or_else(|| DISCONNECTED.clone());
    }
}

impl Default for DirectionalPorts {
    fn default() -> Self {
        Self {
            list: vec![],
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

impl<'a, Message, Renderer> Component<Message, Renderer> for Panel<'a, Message>
where
    Renderer: 'a + iced_native::text::Renderer,
    Renderer::Theme: widget::pick_list::StyleSheet
        + widget::text::StyleSheet
        + widget::scrollable::StyleSheet
        + widget::container::StyleSheet
        + overlay::menu::StyleSheet,
    <Renderer::Theme as overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as widget::pick_list::StyleSheet>::Style>,
{
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

    fn view(&self, _state: &Self::State) -> Element<Event, Renderer> {
        use Direction::*;

        let ports = self.ports.borrow();
        let in_pick_list = pick_list(
            // FIXME optimize the lists clones?
            ports.ins.list.clone(),
            Some(ports.ins.cur.clone()),
            |port| (In, port).into(),
        );
        let out_pick_list = pick_list(
            ports.outs.list.clone(),
            Some(ports.outs.cur.clone()),
            |port| (Out, port).into(),
        );

        // This is to force the labels to occupy the same column
        // whatever the length of the labels. We would need
        // a grid layout here.
        let label_width = Length::Units(40);
        let in_label = text("In:").width(label_width);
        let out_label = text("Out:").width(label_width);

        let content: Element<_, Renderer> = column![
            text("Midi Connection").size(30),
            column![row![in_label, in_pick_list], row![out_label, out_pick_list]].spacing(10),
        ]
        .align_items(Alignment::Fill)
        .spacing(10)
        .into();

        // Uncomment to debug layout
        //content.explain(iced::Color::BLACK)
        content
    }
}

impl<'a, Message, Renderer> From<Panel<'a, Message>> for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + iced_native::text::Renderer,
    Renderer::Theme: widget::pick_list::StyleSheet
        + widget::text::StyleSheet
        + widget::scrollable::StyleSheet
        + widget::container::StyleSheet
        + overlay::menu::StyleSheet,
    <Renderer::Theme as overlay::menu::StyleSheet>::Style:
        From<<Renderer::Theme as widget::pick_list::StyleSheet>::Style>,
{
    fn from(panel: Panel<'a, Message>) -> Self {
        iced_lazy::component(panel)
    }
}
