use iced::{
    widget::{column, pick_list, radio, row, text, toggler, vertical_space},
    Alignment, Element, Length,
};
use iced_lazy::{self, Component};

use crate::{
    jstation::data::{
        dsp::{effect, Effect},
        BoolParameter, ConstRangeParameter, DiscreteParameter, VariableRangeParameter,
    },
    ui::{self, COMBO_TEXT_SIZE, DSP_TITLE_AREA_WIDTH, LABEL_TEXT_SIZE, RADIO_SIZE, RADIO_SPACING},
};

pub struct Panel {
    effect: Effect,
}

impl Panel {
    pub fn new(effect: Effect) -> Self {
        Self { effect }
    }
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<effect::Parameter>,
{
    type State = ();
    type Event = effect::Parameter;

    fn update(&mut self, _state: &mut Self::State, param: effect::Parameter) -> Option<Message> {
        use crate::jstation::data::ParameterSetter;
        self.effect.set(param).map(Message::from)
    }

    fn view(&self, _state: &Self::State) -> Element<effect::Parameter> {
        let title_area = column![
            text("Effect"),
            vertical_space(Length::Units(10)),
            row![
                toggler("".to_string(), self.effect.switch.into(), |is_on| {
                    effect::Parameter::Switch(is_on.into())
                })
                .width(Length::Shrink),
                pick_list(
                    effect::Type::names(),
                    Some(self.effect.typ.name()),
                    |name| name.param().into(),
                )
                .text_size(COMBO_TEXT_SIZE),
                column![
                    radio("Pre", effect::Post::FALSE, Some(self.effect.post), |post| {
                        post.into()
                    },)
                    .size(RADIO_SIZE)
                    .text_size(LABEL_TEXT_SIZE)
                    .spacing(RADIO_SPACING),
                    vertical_space(Length::Units(6)),
                    radio("Post", effect::Post::TRUE, Some(self.effect.post), |post| {
                        post.into()
                    },)
                    .size(RADIO_SIZE)
                    .text_size(LABEL_TEXT_SIZE)
                    .spacing(RADIO_SPACING),
                ],
            ]
            .spacing(15),
        ]
        .width(DSP_TITLE_AREA_WIDTH)
        .padding(5);

        use effect::Parameter::*;
        let mut content = row![
            title_area,
            ui::knob(self.effect.mix, |normal| {
                Mix(effect::Mix::from_normal(normal))
            })
            .build(),
        ]
        .spacing(10)
        .align_items(Alignment::End);

        content = content.push({
            // FIXME could be a pick list for Wah
            ui::knob(self.effect.speed, |normal| {
                Speed(
                    effect::Speed::from_normal(self.effect.typ.into(), normal)
                        .expect("always defined"),
                )
            })
            .name(match self.effect.speed.assignment() {
                effect::SpeedAssignment::Speed => "Speed",
                effect::SpeedAssignment::WahType => "Type",
                effect::SpeedAssignment::Semitones => "½tones",
            })
            .build()
        });

        content = content.push({
            ui::knob(self.effect.depth, |normal| {
                Depth(
                    effect::Depth::from_normal(self.effect.typ.into(), normal)
                        .expect("always defined"),
                )
            })
            .name(match self.effect.depth.assignment() {
                effect::DepthAssignment::Depth => "Depth",
                effect::DepthAssignment::Detune => "Detune",
            })
            .build()
        });

        if self.effect.regen.is_active() {
            content = content.push(
                ui::knob(self.effect.regen, |normal| {
                    Regen(
                        effect::Regen::from_normal(self.effect.typ.into(), normal)
                            .expect("regen is active"),
                    )
                })
                .build(),
            )
        }

        // Set to true to debug layout
        if false {
            Element::from(content).explain(iced::Color::WHITE)
        } else {
            content.into()
        }
    }
}

impl<'a, Message> From<Panel> for Element<'a, Message, iced::Renderer>
where
    Message: 'a + From<effect::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
