use iced::{
    widget::{column, row, text, toggler},
    Alignment, Element, Length,
};
use iced_audio::Knob;
use iced_lazy::{self, Component};

use std::{cell::RefCell, rc::Rc};

use crate::{
    jstation::data::{
        dsp::{noise_gate, NoiseGate, NoiseGateParameter},
        BoolParameter, DiscreteParameter,
    },
    ui::{to_jstation_normal, to_ui_param},
};

const KNOB_SIZE: Length = Length::Units(35);

pub struct Panel<'a, Message> {
    noise_gate: Rc<RefCell<NoiseGate>>,
    on_change: Box<dyn 'a + Fn(NoiseGateParameter) -> Message>,
}

impl<'a, Message> Panel<'a, Message> {
    pub fn new<F>(noise_gate: Rc<RefCell<NoiseGate>>, on_change: F) -> Self
    where
        F: 'a + Fn(NoiseGateParameter) -> Message,
    {
        Self {
            noise_gate,
            on_change: Box::new(on_change),
        }
    }
}

impl<'a, Message> Component<Message, iced::Renderer> for Panel<'a, Message> {
    type State = ();
    type Event = NoiseGateParameter;

    fn update(&mut self, _state: &mut Self::State, event: NoiseGateParameter) -> Option<Message> {
        use NoiseGateParameter::*;
        let changed_param = param_handling!(
            self.noise_gate,
            match event {
                GateOn => gate_on,
                AttackTime => attack_time,
                Threshold => threshold,
            }
        );

        Some((self.on_change)(changed_param))
    }

    fn view(&self, _state: &Self::State) -> Element<NoiseGateParameter> {
        macro_rules! param_knob {
            ($ng:ident, $variant:ident, $param:ident) => {
                column![
                    text(stringify!($variant)),
                    Knob::new(to_ui_param($ng.$param), |normal| {
                        $variant(noise_gate::$variant::from_snapped(to_jstation_normal(
                            normal,
                        )))
                        .into()
                    })
                    .size(KNOB_SIZE),
                    text(format!("{:02}", $ng.$param)),
                ]
                .spacing(5)
                .align_items(Alignment::Center)
            };
        }

        use NoiseGateParameter::*;

        let ng = self.noise_gate.borrow();

        let content: Element<_> = row![
            column![
                text("Noise Gate"),
                toggler("".to_string(), ng.gate_on.is_active(), |is_active| {
                    GateOn(is_active.into())
                })
                .width(Length::Shrink)
            ]
            .spacing(10)
            .align_items(Alignment::Start),
            param_knob!(ng, AttackTime, attack_time),
            param_knob!(ng, Threshold, threshold),
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
