use iced::{
    widget::{horizontal_space, row},
    Alignment, Element, Length,
};
use iced_lazy::{self, Component};

use crate::jstation::{
    data::dsp::{noise_gate, NoiseGate},
    prelude::*,
};
use crate::ui;

pub struct Panel {
    noise_gate: NoiseGate,
}

impl Panel {
    pub fn new(noise_gate: NoiseGate) -> Self {
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
        self.noise_gate.set(event).map(Message::from)
    }

    fn view(&self, _state: &Self::State) -> Element<noise_gate::Parameter> {
        use noise_gate::Parameter::*;
        let content: Element<_> = row![
            ui::switch("Noise Gate", self.noise_gate.switch, |is_on| {
                noise_gate::Switch::from(is_on)
            }),
            horizontal_space(Length::Fixed(25f32)),
            row![
                ui::knob(self.noise_gate.attack_time, |normal| AttackTime(
                    noise_gate::AttackTime::from_normal(normal)
                ))
                .name("Attack")
                .build(),
                horizontal_space(Length::Fixed(10f32)),
                ui::knob(self.noise_gate.threshold, |normal| Threshold(
                    noise_gate::Threshold::from_normal(normal)
                ))
                .name("Thold")
                .build(),
            ]
            .align_items(Alignment::End),
        ]
        .width(Length::Fixed(230f32))
        .height(Length::Fixed(78f32))
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
