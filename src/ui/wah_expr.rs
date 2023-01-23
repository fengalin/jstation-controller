use iced::{
    widget::{column, horizontal_space, row, text, vertical_space},
    Element, Length,
};
use iced_lazy::{self, Component};

use crate::jstation::{
    data::dsp::{
        self, {expression, pedal, wah, Expression, Pedal, Wah},
    },
    prelude::*,
};
use crate::ui;

pub struct Panel {
    expression: Expression,
    pedal: Pedal,
    wah: Wah,
}

impl Panel {
    pub fn new(expression: Expression, pedal: Pedal, wah: Wah) -> Self {
        Self {
            expression,
            pedal,
            wah,
        }
    }
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<dsp::Parameter>,
{
    type State = ();
    type Event = dsp::Parameter;

    fn update(&mut self, _state: &mut Self::State, event: dsp::Parameter) -> Option<Message> {
        use dsp::Parameter::*;
        match event {
            Expression(param) => self.expression.set(param).map(dsp::Parameter::from),
            Pedal(param) => self.pedal.set(param).map(dsp::Parameter::from),
            Wah(param) => self.wah.set(param).map(dsp::Parameter::from),
            _ => unreachable!(),
        }
        .map(Message::from)
    }

    fn view(&self, _state: &Self::State) -> Element<dsp::Parameter> {
        use dsp::Parameter::*;

        let mut selection = row![column![
            vertical_space(Length::Units(3)),
            ui::radio(
                "Expr.",
                wah::Switch::FALSE,
                Some(self.wah.switch),
                |is_wah| Wah(is_wah.into())
            ),
            vertical_space(Length::Units(6)),
            ui::radio("Wah", wah::Switch::TRUE, Some(self.wah.switch), |is_wah| {
                Wah(is_wah.into())
            }),
        ]];

        let mut pedal = if self.wah.switch.is_true() {
            row![
                ui::knob(self.wah.heel, |normal| Wah(
                    wah::Heel::from_normal(normal).into()
                ))
                .build(),
                ui::knob(self.wah.toe, |normal| {
                    Wah(wah::Toe::from_normal(normal).into())
                })
                .build(),
            ]
        } else {
            selection = selection.push(horizontal_space(Length::Units(10)));
            selection = selection.push(ui::pick_list(
                expression::Assignment::names(),
                Some(self.expression.assignment.name()),
                |name| Expression(name.param().into()),
            ));

            row![
                ui::knob(self.expression.back, |normal| {
                    Expression(expression::Back::from_normal(normal).into())
                })
                .build(),
                ui::knob(self.expression.forward, |normal| Expression(
                    expression::Forward::from_normal(normal).into()
                ))
                .name("Fwd")
                .build(),
            ]
        };

        pedal = pedal.push(column![
            ui::label(if self.wah.switch.is_true() {
                "Wah"
            } else {
                "Expression"
            }),
            vertical_space(Length::Units(2)),
            ui::hslider(self.pedal.expression, |normal| Pedal(
                pedal::Expression::from_normal(normal).into()
            )),
            vertical_space(Length::Units(7)),
            ui::label("Volume"),
            vertical_space(Length::Units(2)),
            ui::hslider(self.pedal.volume, |normal| Pedal(
                pedal::Volume::from_normal(normal).into()
            )),
        ]);

        let content: Element<_> = ui::dsp(
            column![text("Pedal"), vertical_space(Length::Units(10)), selection],
            pedal.spacing(10),
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
    Message: 'a + From<dsp::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
