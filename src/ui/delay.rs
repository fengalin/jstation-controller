use iced::{
    widget::{column, pick_list, row, text, toggler, vertical_space},
    Element, Length,
};
use iced_lazy::{self, Component};

use crate::jstation::data::{
    dsp::{delay, Delay},
    ConstRangeParameter,
};
use crate::ui::{self, COMBO_TEXT_SIZE};
pub struct Panel {
    delay: Delay,
}

impl Panel {
    pub fn new(delay: Delay) -> Self {
        Self { delay }
    }
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<delay::Parameter>,
{
    type State = ();
    type Event = delay::Parameter;

    fn update(&mut self, _state: &mut Self::State, event: delay::Parameter) -> Option<Message> {
        use crate::jstation::data::ParameterSetter;
        self.delay.set(event).map(Message::from)
    }

    fn view(&self, _state: &Self::State) -> Element<delay::Parameter> {
        use delay::Parameter::*;

        let title_area = column![
            text("Delay"),
            vertical_space(Length::Units(10)),
            row![
                toggler("".to_string(), self.delay.switch.into(), |is_on| {
                    delay::Parameter::Switch(is_on.into())
                })
                .width(Length::Shrink),
                pick_list(delay::Type::names(), Some(self.delay.typ.name()), |name| {
                    name.param().into()
                },)
                .text_size(COMBO_TEXT_SIZE),
            ]
            .spacing(15),
        ];

        let content: Element<_> = ui::dsp(
            title_area,
            row![
                ui::knob(self.delay.level, |normal| {
                    Level(delay::Level::from_normal(normal))
                })
                .build(),
                ui::knob(self.delay.time_course, |normal| TimeCourse(
                    delay::TimeCourse::from_normal(normal)
                ))
                .name("Course")
                .build(),
                ui::knob(self.delay.time_fine, |normal| TimeFine(
                    delay::TimeFine::from_normal(normal)
                ))
                .name("Fine")
                .build(),
                ui::knob(self.delay.feedback, |normal| {
                    Feedback(delay::Feedback::from_normal(normal))
                })
                .name("Fback")
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
    Message: 'a + From<delay::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
