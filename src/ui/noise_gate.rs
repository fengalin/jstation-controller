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
        let noise_gate = self.noise_gate.borrow();

        use noise_gate::Parameter::*;
        let content: Element<_> = row![
            param_switch!("Noise Gate", noise_gate, switch, Switch),
            param_knob!(noise_gate, attack_time, AttackTime),
            param_knob!(noise_gate, threshold, Threshold),
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
