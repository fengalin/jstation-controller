use iced::{
    widget::{column, horizontal_space, row, text, vertical_space},
    Alignment, Element, Length,
};
use iced_lazy::{self, Component};

use crate::jstation::data::{
    dsp::{amp, Amp},
    ConstRangeParameter, DiscreteParameter,
};
use crate::ui;

#[derive(Debug, Clone)]
pub enum Event {
    Parameter(amp::Parameter),
    MustShowNicks(bool),
}

impl From<amp::Parameter> for Event {
    fn from(param: amp::Parameter) -> Self {
        Event::Parameter(param)
    }
}

pub struct Panel {
    amp: Amp,
}

impl Panel {
    pub fn new(amp: Amp) -> Self {
        Self { amp }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    show_nick: bool,
}

impl<Message> Component<Message, iced::Renderer> for Panel
where
    Message: From<amp::Parameter>,
{
    type State = State;
    type Event = Event;

    fn update(&mut self, state: &mut Self::State, event: Event) -> Option<Message> {
        use Event::*;
        match event {
            Parameter(param) => {
                use crate::jstation::data::ParameterSetter;
                self.amp.set(param).map(Message::from)
            }
            MustShowNicks(show_nick) => {
                state.show_nick = show_nick;
                None
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<Event> {
        let mut modeling = column![
            row![
                ui::amp_cabinet_label(self.amp.modeling.param_name()),
                ui::checkbox("nick", state.show_nick, Event::MustShowNicks),
            ],
            vertical_space(Length::Units(5)),
        ];

        if state.show_nick {
            modeling = modeling.push(ui::pick_list(
                amp::Modeling::nicks(),
                Some(self.amp.modeling.nick()),
                |nick| amp::Parameter::from(nick.param()).into(),
            ));
        } else {
            modeling = modeling.push(ui::pick_list(
                amp::Modeling::names(),
                Some(self.amp.modeling.name()),
                |name| amp::Parameter::from(name.param()).into(),
            ));
        }

        let title_area = column![text("Amp"), vertical_space(Length::Units(10)), modeling];

        use amp::Parameter::*;
        let content: Element<_> = ui::dsp(
            title_area,
            row![
                ui::knob(self.amp.gain, |normal| Gain(amp::Gain::from_normal(normal))).build(),
                horizontal_space(Length::Units(15)),
                ui::knob(self.amp.bass, |normal| Bass(amp::Bass::from_normal(normal))).build(),
                horizontal_space(Length::Units(10)),
                ui::knob(self.amp.middle, |normal| Middle(amp::Middle::from_normal(
                    normal
                )))
                .build(),
                horizontal_space(Length::Units(10)),
                ui::knob(self.amp.treble, |normal| Treble(amp::Treble::from_normal(
                    normal
                )))
                .build(),
                horizontal_space(Length::Units(15)),
                ui::knob(self.amp.level, |normal| Level(amp::Level::from_normal(
                    normal
                )))
                .build(),
            ]
            .align_items(Alignment::End),
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
    Message: 'a + From<amp::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
