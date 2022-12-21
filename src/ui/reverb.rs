use iced::{
    widget::{column, pick_list, row, text, toggler, vertical_space},
    Element, Length,
};
use iced_lazy::{self, Component};

use crate::{
    jstation::data::{
        dsp::{reverb, Reverb},
        ConstRangeParameter,
    },
    ui::{self, COMBO_TEXT_SIZE, DSP_TITLE_AREA_WIDTH},
};

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
                toggler("".to_string(), self.reverb.switch.into(), |is_on| {
                    reverb::Parameter::Switch(is_on.into())
                })
                .width(Length::Shrink),
                pick_list(
                    reverb::Type::names(),
                    Some(self.reverb.typ.name()),
                    |name| { name.param().into() },
                )
                .text_size(COMBO_TEXT_SIZE),
            ]
            .spacing(15),
        ]
        .width(DSP_TITLE_AREA_WIDTH)
        .padding(5);

        let content: Element<_> = row![
            title_area,
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
        .spacing(10)
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
