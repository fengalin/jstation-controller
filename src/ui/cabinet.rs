use iced::{
    widget::{checkbox, column, pick_list, row, text, vertical_space},
    Element, Length,
};
use iced_lazy::{self, Component};

use crate::{
    jstation::data::{
        dsp::{cabinet, Cabinet},
        DiscreteParameter,
    },
    ui::{AMP_CABINET_LABEL_WIDTH, CHECKBOX_SIZE, COMBO_TEXT_SIZE, LABEL_TEXT_SIZE},
};

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
    cabinet: Cabinet,
}

impl Panel {
    pub fn new(cabinet: Cabinet) -> Self {
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

                return self.cabinet.set(cabinet_param).map(Message::from);
            }
            MustShowNicks(show_nick) => state.show_nick = show_nick,
        }

        None
    }

    fn view(&self, state: &Self::State) -> Element<Event> {
        let mut cabinet_types = column![
            text("Cabinet"),
            vertical_space(Length::Units(10)),
            row![
                text(self.cabinet.typ.param_name())
                    .size(LABEL_TEXT_SIZE)
                    .width(AMP_CABINET_LABEL_WIDTH),
                checkbox("nick", state.show_nick, Event::MustShowNicks).size(CHECKBOX_SIZE),
            ],
            vertical_space(Length::Units(5)),
        ]
        .width(Length::Units(350))
        .padding(5);

        if state.show_nick {
            cabinet_types = cabinet_types.push(
                pick_list(
                    cabinet::Type::nicks(),
                    Some(self.cabinet.typ.nick()),
                    |nick| nick.param().into(),
                )
                .text_size(COMBO_TEXT_SIZE),
            );
        } else {
            cabinet_types = cabinet_types.push(
                pick_list(
                    cabinet::Type::names(),
                    Some(self.cabinet.typ.name()),
                    |name| name.param().into(),
                )
                .text_size(COMBO_TEXT_SIZE),
            );
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
