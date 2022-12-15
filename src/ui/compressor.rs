use iced::{
    widget::{column, row, text, toggler},
    Alignment, Element, Length,
};
use iced_audio::Knob;
use iced_lazy::{self, Component};

use std::{cell::RefCell, rc::Rc};

use crate::{
    jstation::data::{
        dsp::{compressor, Compressor},
        BoolParameter, DiscreteParameter,
    },
    ui::{to_jstation_normal, to_ui_param},
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
            param_switch!("Compressor", compressor, switch, Switch),
            param_knob!(compressor, threshold, Threshold),
            param_knob!(compressor, ratio, Ratio, compressor.ratio.value()),
            param_knob!(compressor, gain, Gain),
            param_knob!(compressor, freq, Freq, compressor.freq.value()),
        ]
        .spacing(10)
        .align_items(Alignment::Fill)
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
