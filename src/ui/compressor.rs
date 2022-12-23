use iced::{widget::row, Element};
use iced_lazy::{self, Component};

use crate::jstation::data::{
    dsp::{compressor, Compressor},
    ConstRangeParameter,
};
use crate::ui;

pub struct Panel {
    compressor: Compressor,
}

impl Panel {
    pub fn new(compressor: Compressor) -> Self {
        Self { compressor }
    }
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<compressor::Parameter>,
{
    type State = ();
    type Event = compressor::Parameter;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: compressor::Parameter,
    ) -> Option<Message> {
        use crate::jstation::data::ParameterSetter;
        self.compressor.set(event).map(Message::from)
    }

    fn view(&self, _state: &Self::State) -> Element<compressor::Parameter> {
        use compressor::Parameter::*;

        let content: Element<_> = ui::dsp(
            ui::switch("Compressor", self.compressor.switch, |is_on| {
                compressor::Switch::from(is_on)
            }),
            row![
                ui::knob(self.compressor.threshold, |normal| Threshold(
                    compressor::Threshold::from_normal(normal)
                ))
                .name("Thold")
                .build(),
                ui::knob(self.compressor.ratio, |normal| {
                    Ratio(compressor::Ratio::from_normal(normal))
                })
                .build(),
                ui::knob(self.compressor.gain, |normal| Gain(
                    compressor::Gain::from_normal(normal)
                ))
                .build(),
                ui::knob(self.compressor.freq, |normal| {
                    Freq(compressor::Freq::from_normal(normal))
                })
                .name("Freq")
                .build(),
            ]
            .spacing(10),
        )
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
    Message: 'a + From<compressor::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
