use iced::{
    widget::{column, row, text, toggler},
    Alignment, Element, Length,
};
use iced_audio::Knob;
use iced_lazy::{self, Component};

use std::{cell::RefCell, rc::Rc};

use crate::{
    jstation::data::{
        dsp::{noise_gate, NoiseGate},
        BoolParameter, DiscreteParameter,
    },
    ui::{to_jstation_normal, to_ui_param},
};

const KNOB_SIZE: Length = Length::Units(35);

pub struct Panel {
    noise_gate: Rc<RefCell<NoiseGate>>,
}

impl Panel {
    pub fn new(noise_gate: Rc<RefCell<NoiseGate>>) -> Self {
        Self { noise_gate }
    }
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<noise_gate::Parameter>,
{
    type State = ();
    type Event = noise_gate::Parameter;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: noise_gate::Parameter,
    ) -> Option<Message> {
        use crate::jstation::data::ParameterSetter;
        self.noise_gate.borrow_mut().set(event).map(Message::from)
    }

    fn view(&self, _state: &Self::State) -> Element<noise_gate::Parameter> {
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

        let ng = self.noise_gate.borrow();

        use noise_gate::Parameter::*;
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

impl<'a, Message> From<Panel> for Element<'a, Message, iced::Renderer>
where
    Message: 'a + From<noise_gate::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
