use iced::{
    widget::{column, row, text, vertical_space},
    Alignment, Element, Length,
};
use iced_lazy::{self, Component};

use crate::jstation::data::{
    dsp::{effect, Effect},
    BoolParameter, ConstRangeParameter, DiscreteParameter, VariableRangeParameter,
};
use crate::ui;

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
                ui::toggler(self.effect.switch.into(), |is_on| {
                    effect::Parameter::Switch(is_on.into())
                }),
                ui::pick_list(
                    effect::Type::names(),
                    Some(self.effect.typ.name()),
                    |name| name.param().into(),
                ),
                column![
                    ui::radio("Pre", effect::Post::FALSE, Some(self.effect.post), |post| {
                        post.into()
                    }),
                    vertical_space(Length::Units(6)),
                    ui::radio("Post", effect::Post::TRUE, Some(self.effect.post), |post| {
                        post.into()
                    }),
                ],
            ]
            .spacing(15),
        ];

        use effect::Parameter::*;
        let mut content = row![ui::knob(self.effect.mix, |normal| {
            Mix(effect::Mix::from_normal(normal))
        })
        .build(),]
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

        let content = ui::dsp(title_area, content);

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
