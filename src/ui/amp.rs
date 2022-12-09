use iced::{
    widget::{checkbox, column, pick_list, row, text},
    Alignment, Element, Length,
};
use iced_audio::Knob;
use iced_lazy::{self, Component};

use std::{cell::RefCell, rc::Rc};

use crate::{
    jstation::data::{
        dsp::{amp, Amp, AmpParameter},
        DiscreteParameter,
    },
    ui::{to_jstation_normal, to_ui_param},
};

const KNOB_SIZE: Length = Length::Units(35);

#[derive(Debug, Clone)]
pub enum Event {
    Parameter(AmpParameter),
    MustShowAltNames(bool),
}

impl From<AmpParameter> for Event {
    fn from(param: AmpParameter) -> Self {
        Event::Parameter(param)
    }
}

pub struct Panel<'a, Message> {
    amp: Rc<RefCell<Amp>>,
    on_change: Box<dyn 'a + Fn(AmpParameter) -> Message>,
}

impl<'a, Message> Panel<'a, Message> {
    pub fn new<F>(amp: Rc<RefCell<Amp>>, on_change: F) -> Self
    where
        F: 'a + Fn(AmpParameter) -> Message,
    {
        Self {
            amp,
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
            Parameter(param) => {
                use AmpParameter::*;
                let changed_param = param_handling!(
                    self.amp,
                    match param {
                        Modeling => modeling,
                        Gain => gain,
                        Treble => treble,
                        Middle => middle,
                        Bass => bass,
                        Level => level,
                    }
                );

                return Some((self.on_change)(changed_param));
            }
            MustShowAltNames(show_nick) => state.show_nick = show_nick,
        }

        None
    }

    fn view(&self, state: &Self::State) -> Element<Event> {
        macro_rules! param_knob {
            ($amp:ident, $variant:ident, $param:ident) => {
                column![
                    text(stringify!($variant)),
                    Knob::new(to_ui_param($amp.$param), |normal| {
                        $variant(amp::$variant::from_snapped(to_jstation_normal(normal))).into()
                    })
                    .size(KNOB_SIZE),
                    text(format!("{:02}", $amp.$param)),
                ]
                .spacing(5)
                .align_items(Alignment::Center)
            };
        }

        use AmpParameter::*;

        let amp = self.amp.borrow();

        let mut modelings = column![row![
            text("Amplifier Model"),
            checkbox("nick", state.show_nick, Event::MustShowAltNames),
        ]
        .spacing(10),]
        .spacing(10)
        .padding(5);

        if state.show_nick {
            modelings = modelings.push(pick_list(
                amp::Modeling::nicks(),
                Some(amp.modeling.nick()),
                |nick| Modeling(nick.param()).into(),
            ));
        } else {
            modelings = modelings.push(pick_list(
                amp::Modeling::names(),
                Some(amp.modeling.name()),
                |name| Modeling(name.param()).into(),
            ));
        }

        let content: Element<_> = row![
            modelings,
            param_knob!(amp, Gain, gain),
            param_knob!(amp, Treble, treble),
            param_knob!(amp, Middle, middle),
            param_knob!(amp, Bass, bass),
            param_knob!(amp, Level, level),
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

impl<'a, Message: 'a> From<Panel<'a, Message>> for Element<'a, Message, iced::Renderer> {
    fn from(panel: Panel<'a, Message>) -> Self {
        iced_lazy::component(panel)
    }
}
