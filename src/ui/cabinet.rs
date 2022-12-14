use iced::{
    widget::{checkbox, column, pick_list, row, text},
    Element,
};
use iced_lazy::{self, Component};

use std::{cell::RefCell, rc::Rc};

use crate::jstation::data::dsp::{cabinet, Cabinet};

#[derive(Debug, Clone)]
pub enum Event {
    Parameter(cabinet::Parameter),
    MustShowNicks(bool),
}

impl From<cabinet::Type> for Event {
    fn from(param: cabinet::Type) -> Self {
        Event::Parameter(cabinet::Parameter::Type(param))
    }
}

pub struct Panel {
    cabinet: Rc<RefCell<Cabinet>>,
}

impl Panel {
    pub fn new(cabinet: Rc<RefCell<Cabinet>>) -> Self {
        Self { cabinet }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    show_nick: bool,
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<cabinet::Parameter>,
{
    type State = State;
    type Event = Event;

    fn update(&mut self, state: &mut Self::State, event: Event) -> Option<Message> {
        use Event::*;
        match event {
            Parameter(cabinet_param) => {
                use crate::jstation::data::ParameterSetter;

                return self
                    .cabinet
                    .borrow_mut()
                    .set(cabinet_param)
                    .map(Message::from);
            }
            MustShowNicks(show_nick) => state.show_nick = show_nick,
        }

        None
    }

    fn view(&self, state: &Self::State) -> Element<Event> {
        let cabinet = self.cabinet.borrow();

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
                Some(cabinet.typ.nick()),
                |nick| nick.param().into(),
            ));
        } else {
            cabinet_types = cabinet_types.push(pick_list(
                cabinet::Type::names(),
                Some(cabinet.typ.name()),
                |name| name.param().into(),
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

impl<'a, Message> From<Panel> for Element<'a, Message, iced::Renderer>
where
    Message: 'a + From<cabinet::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
