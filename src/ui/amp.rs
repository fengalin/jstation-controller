use iced::{
    widget::{checkbox, column, pick_list, row, text},
    Alignment, Element,
};
use iced_audio::Knob;
use iced_lazy::{self, Component};

use std::{cell::RefCell, rc::Rc};

use crate::{
    jstation::data::{
        dsp::{amp, Amp},
        DiscreteParameter,
    },
    ui::{to_jstation_normal, to_ui_param},
};

#[derive(Debug, Clone)]
pub enum Event {
    Parameter(amp::Parameter),
    MustShowNicks(bool),
}

impl From<amp::Parameter> for Event {
    fn from(param: amp::Parameter) -> Self {
        Event::Parameter(param)
    }
}

pub struct Panel {
    amp: Rc<RefCell<Amp>>,
}

impl Panel {
    pub fn new(amp: Rc<RefCell<Amp>>) -> Self {
        Self { amp }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    show_nick: bool,
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<amp::Parameter>,
{
    type State = State;
    type Event = Event;

    fn update(&mut self, state: &mut Self::State, event: Event) -> Option<Message> {
        use Event::*;
        match event {
            Parameter(param) => {
                use crate::jstation::data::ParameterSetter;
                return self.amp.borrow_mut().set(param).map(Message::from);
            }
            MustShowNicks(show_nick) => {
                state.show_nick = show_nick;
                None
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<Event> {
        let amp = self.amp.borrow();

        let mut modelings = column![row![
            text(amp::Modeling::NAME),
            checkbox("nick", state.show_nick, Event::MustShowNicks),
        ]
        .spacing(10),]
        .spacing(10)
        .padding(5);

        if state.show_nick {
            modelings = modelings.push(pick_list(
                amp::Modeling::nicks(),
                Some(amp.modeling.nick()),
                |nick| amp::Parameter::from(nick.param()).into(),
            ));
        } else {
            modelings = modelings.push(pick_list(
                amp::Modeling::names(),
                Some(amp.modeling.name()),
                |name| amp::Parameter::from(name.param()).into(),
            ));
        }

        use amp::Parameter::*;
        let content: Element<_> = row![
            modelings,
            param_knob!(amp, gain, Gain),
            param_knob!(amp, bass, Bass),
            param_knob!(amp, middle, Middle),
            param_knob!(amp, treble, Treble),
            param_knob!(amp, level, Level),
        ]
        .spacing(10)
        .align_items(Alignment::Fill)
        .into();

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
    Message: 'a + From<amp::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
