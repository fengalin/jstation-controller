use iced::{
    widget::{checkbox, column, pick_list, row, text},
    Element,
};
use iced_lazy::{self, Component};

use std::{cell::RefCell, rc::Rc};

use crate::jstation::data::{dsp::cabinet, DiscreteParameter};

#[derive(Debug, Clone)]
pub enum Event {
    Parameter(cabinet::Type),
    MustShowNicks(bool),
}

impl From<cabinet::Type> for Event {
    fn from(param: cabinet::Type) -> Self {
        Event::Parameter(param)
    }
}

pub struct Panel<'a, Message> {
    cabinet: Rc<RefCell<cabinet::Type>>,
    on_change: Box<dyn 'a + Fn(cabinet::Type) -> Message>,
}

impl<'a, Message> Panel<'a, Message> {
    pub fn new<F>(cabinet: Rc<RefCell<cabinet::Type>>, on_change: F) -> Self
    where
        F: 'a + Fn(cabinet::Type) -> Message,
    {
        Self {
            cabinet,
            on_change: Box::new(on_change),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    show_nick: bool,
}

impl<'a, Message> Component<Message, iced::Renderer> for Panel<'a, Message> {
    type State = State;
    type Event = Event;

    fn update(&mut self, state: &mut Self::State, event: Event) -> Option<Message> {
        use Event::*;
        match event {
            Parameter(cabinet_type) => {
                let param = &mut self.cabinet.borrow_mut();
                if param.set(cabinet_type).is_unchanged() {
                    return None;
                }

                return Some((self.on_change)(param.to_owned()));
            }
            MustShowNicks(show_nick) => state.show_nick = show_nick,
        }

        None
    }

    fn view(&self, state: &Self::State) -> Element<Event> {
        let cabinet_type = self.cabinet.borrow();

        let mut cabinet_types = column![row![
            text("Cabinet type"),
            checkbox("nick", state.show_nick, Event::MustShowNicks),
        ]
        .spacing(10),]
        .spacing(10)
        .padding(5);

        if state.show_nick {
            cabinet_types = cabinet_types.push(pick_list(
                cabinet::Type::nicks(),
                Some(cabinet_type.nick()),
                |nick| Event::Parameter(nick.param()),
            ));
        } else {
            cabinet_types = cabinet_types.push(pick_list(
                cabinet::Type::names(),
                Some(cabinet_type.name()),
                |name| Event::Parameter(name.param()),
            ));
        }

        let content: Element<_> = cabinet_types.into();

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
