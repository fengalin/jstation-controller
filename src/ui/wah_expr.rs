use iced::{
    widget::{column, horizontal_space, pick_list, row, text, toggler, vertical_space},
    Element, Length,
};
use iced_lazy::{self, Component};

use crate::jstation::data::{
    dsp::{wah_expr, WahExpr},
    ConstRangeParameter,
};
use crate::ui::{self, COMBO_TEXT_SIZE};

pub struct Panel {
    wah_expr: WahExpr,
}

impl Panel {
    pub fn new(wah_expr: WahExpr) -> Self {
        Self { wah_expr }
    }
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<wah_expr::Parameter>,
{
    type State = ();
    type Event = wah_expr::Parameter;

    fn update(&mut self, _state: &mut Self::State, event: wah_expr::Parameter) -> Option<Message> {
        use crate::jstation::data::ParameterSetter;
        self.wah_expr.set(event).map(Message::from)
    }

    fn view(&self, _state: &Self::State) -> Element<wah_expr::Parameter> {
        use wah_expr::Parameter::*;

        let title_area = column![
            text("Wah / Expression"),
            vertical_space(Length::Units(10)),
            row![
                toggler("".to_string(), self.wah_expr.switch.into(), |is_on| {
                    wah_expr::Parameter::Switch(is_on.into())
                })
                .width(Length::Shrink),
                horizontal_space(Length::Units(15)),
                pick_list(
                    wah_expr::Assignment::names(),
                    Some(self.wah_expr.assignment.name()),
                    |name| name.param().into(),
                )
                .text_size(COMBO_TEXT_SIZE),
            ],
        ];

        let content: Element<_> = ui::dsp(
            title_area,
            row![
                ui::knob(self.wah_expr.heel, |normal| Heel(
                    wah_expr::Heel::from_normal(normal)
                ))
                .build(),
                ui::knob(self.wah_expr.toe, |normal| {
                    Toe(wah_expr::Toe::from_normal(normal))
                })
                .build(),
                ui::knob(self.wah_expr.forward, |normal| Forward(
                    wah_expr::Forward::from_normal(normal)
                ))
                .name("Fwd")
                .build(),
                ui::knob(self.wah_expr.back, |normal| {
                    Back(wah_expr::Back::from_normal(normal))
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
    Message: 'a + From<wah_expr::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
