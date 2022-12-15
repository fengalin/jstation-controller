use std::{cell::RefCell, rc::Rc};

use iced::{
    widget::{column, horizontal_space, row, text, toggler},
    Alignment, Element, Length,
};
use iced_audio::Knob;
use iced_lazy::{self, Component};

use crate::{
    jstation::data::{
        dsp::{compressor, Compressor},
        BoolParameter, DiscreteParameter,
    },
    ui::{to_jstation_normal, to_ui_param, DSP_TITLE_AREA_WIDTH},
};

pub struct Panel {
    compressor: Rc<RefCell<Compressor>>,
}

impl Panel {
    pub fn new(compressor: Rc<RefCell<Compressor>>) -> Self {
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
        self.compressor.borrow_mut().set(event).map(Message::from)
    }

    fn view(&self, _state: &Self::State) -> Element<compressor::Parameter> {
        let compressor = self.compressor.borrow();

        use compressor::Parameter::*;
        let content: Element<_> = row![
            param_switch!(@name "Compressor", compressor, switch, Switch)
                .width(DSP_TITLE_AREA_WIDTH),
            param_knob!(@name "Thold", compressor, threshold, Threshold),
            horizontal_space(Length::Units(2)),
            param_knob!(compressor, ratio, Ratio, compressor.ratio.value()),
            param_knob!(compressor, gain, Gain),
            param_knob!(@name "Freq.",compressor, freq, Freq, compressor.freq.value()),
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
    Message: 'a + From<compressor::Parameter>,
{
    fn from(panel: Panel) -> Self {
        iced_lazy::component(panel)
    }
}
