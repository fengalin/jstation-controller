use std::{cell::RefCell, rc::Rc};

use iced::{
    widget::{checkbox, column, horizontal_space, pick_list, row, text, vertical_space},
    Alignment, Element, Length,
};
use iced_audio::Knob;
use iced_lazy::{self, Component};

use crate::{
    jstation::data::{
        dsp::{amp, Amp},
        DiscreteParameter,
    },
    ui::{
        to_jstation_normal, to_ui_param, AMP_CABINET_LABEL_WIDTH, CHECKBOX_SIZE, COMBO_TEXT_SIZE,
        DSP_TITLE_AREA_WIDTH, LABEL_TEXT_SIZE,
    },
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

        let mut modelings = column![
            text("Amp"),
            vertical_space(Length::Units(1)),
            row![
                text(amp::Modeling::NAME)
                    .size(LABEL_TEXT_SIZE)
                    .width(AMP_CABINET_LABEL_WIDTH),
                checkbox("nick", state.show_nick, Event::MustShowNicks).size(CHECKBOX_SIZE),
            ],
        ]
        .width(DSP_TITLE_AREA_WIDTH)
        .spacing(5)
        .padding(5);

        if state.show_nick {
            modelings = modelings.push(
                pick_list(amp::Modeling::nicks(), Some(amp.modeling.nick()), |nick| {
                    amp::Parameter::from(nick.param()).into()
                })
                .text_size(COMBO_TEXT_SIZE),
            );
        } else {
            modelings = modelings.push(
                pick_list(amp::Modeling::names(), Some(amp.modeling.name()), |name| {
                    amp::Parameter::from(name.param()).into()
                })
                .text_size(COMBO_TEXT_SIZE),
            );
        }

        use amp::Parameter::*;
        let content: Element<_> = row![
            modelings,
            param_knob!(amp, gain, Gain),
            horizontal_space(Length::Units(2)),
            param_knob!(amp, bass, Bass),
            param_knob!(amp, middle, Middle),
            param_knob!(amp, treble, Treble),
            horizontal_space(Length::Units(2)),
            param_knob!(amp, level, Level),
        ]
        .spacing(10)
        .align_items(Alignment::End)
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
