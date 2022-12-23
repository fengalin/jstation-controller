use iced::{
    widget::{column, row, text, vertical_space},
    Element, Length,
};
use iced_lazy::{self, Component};

use crate::jstation::data::{
    dsp::{reverb, Reverb},
    ConstRangeParameter,
};
use crate::ui;

pub struct Panel {
    reverb: Reverb,
}

impl Panel {
    pub fn new(reverb: Reverb) -> Self {
        Self { reverb }
    }
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<reverb::Parameter>,
{
    type State = ();
    type Event = reverb::Parameter;

    fn update(&mut self, _state: &mut Self::State, event: reverb::Parameter) -> Option<Message> {
        use crate::jstation::data::ParameterSetter;
        self.reverb.set(event).map(Message::from)
    }

    fn view(&self, _state: &Self::State) -> Element<reverb::Parameter> {
        use reverb::Parameter::*;

        let title_area = column![
            text("Reverb"),
            vertical_space(Length::Units(10)),
            row![
                ui::toggler(self.reverb.switch.into(), |is_on| {
                    reverb::Parameter::Switch(is_on.into())
                }),
                ui::pick_list(
                    reverb::Type::names(),
                    Some(self.reverb.typ.name()),
                    |name| { name.param().into() },
                ),
            ]
            .spacing(15),
        ];

        let content: Element<_> = ui::dsp(
            title_area,
            row![
                ui::knob(self.reverb.level, |normal| {
                    Level(reverb::Level::from_normal(normal))
                })
                .build(),
                ui::knob(self.reverb.diffusion, |normal| Diffusion(
                    reverb::Diffusion::from_normal(normal)
                ))
                .name("Diff.")
                .build(),
                ui::knob(self.reverb.density, |normal| Density(
                    reverb::Density::from_normal(normal)
                ))
                .build(),
                ui::knob(self.reverb.decay, |normal| {
                    Decay(reverb::Decay::from_normal(normal))
                })
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
    Message: 'a + From<reverb::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
