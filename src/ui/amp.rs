use iced::{
    widget::{column, row, text},
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

impl<'a, Message> Component<Message, iced::Renderer> for Panel<'a, Message> {
    type State = ();
    type Event = AmpParameter;

    fn update(&mut self, _state: &mut Self::State, event: AmpParameter) -> Option<Message> {
        use AmpParameter::*;
        let changed_param = match event {
            // FIXME looks like it could be generated
            Modeling(value) => {
                let modeling = &mut self.amp.borrow_mut().modeling;
                if modeling.set(value).is_unchanged() {
                    return None;
                }

                modeling.into()
            }
            Gain(value) => {
                let gain = &mut self.amp.borrow_mut().gain;
                if gain.set(value).is_unchanged() {
                    return None;
                }

                gain.into()
            }
            other => {
                dbg!(&other);
                return None;
            }
        };

        Some((self.on_change)(changed_param))
    }

    fn view(&self, _state: &Self::State) -> Element<AmpParameter> {
        use AmpParameter::*;

        let amp = self.amp.borrow();
        let content: Element<_> = row![
            column![text("Amplifier Model"), text(amp.modeling)]
                .spacing(10)
                .padding(5),
            column![
                text("Gain"),
                row![
                    Knob::new(to_ui_param(amp.gain), |normal| {
                        Gain(amp::Gain::from_snapped(to_jstation_normal(normal))).into()
                    })
                    .size(KNOB_SIZE),
                    text(format!("{:02}", amp.gain)),
                ]
                .spacing(5)
                .align_items(Alignment::Center),
            ]
            .align_items(Alignment::Center),
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
