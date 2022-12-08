use iced::{
    widget::{checkbox, column, row, text},
    Alignment, Element, Length,
};
use iced_audio::{Knob, Normal};
use iced_lazy::{self, Component};

use crate::{
    jstation::data::dsp::{DigitalOutLevel, UtilitySettings},
    ui::{jstation_to_ui_param, ui_to_jstation_normal},
};

const KNOB_SIZE: Length = Length::Units(35);

#[derive(Debug, Clone)]
pub enum Event {
    UtilitySettings(UtilitySettings),
    DigitalOutLevel(DigitalOutLevel),
}

#[derive(Debug)]
pub enum PrivEvent {
    Stereo(bool),
    DryTrack(bool),
    DigitalOutLevel(Normal),
    GlobalCabinet(bool),
    MidiMerge(bool),
    MidiChannel(Normal),
    MidiChannelReleased,
}

pub struct Panel<'a, Message> {
    settings: UtilitySettings,
    digital_out_level_str: String,
    midi_channel_str: String,
    midi_channel_changed: bool,
    on_change: Box<dyn 'a + Fn(Event) -> Message>,
}

impl<'a, Message> Panel<'a, Message> {
    pub fn new<F>(settings: UtilitySettings, on_change: F) -> Self
    where
        F: 'a + Fn(Event) -> Message,
    {
        Self {
            settings,
            digital_out_level_str: format!("{:02}", settings.digital_out_level),
            midi_channel_str: format!("{:02}", settings.midi_channel),
            midi_channel_changed: false,
            on_change: Box::new(on_change),
        }
    }
}

impl<'a, Message> Component<Message, iced::Renderer> for Panel<'a, Message> {
    type State = ();
    type Event = PrivEvent;

    fn update(&mut self, _state: &mut Self::State, event: PrivEvent) -> Option<Message> {
        use crate::jstation::data::DiscreteParameter;

        use PrivEvent::*;
        match event {
            Stereo(is_checked) => self.settings.stereo_mono = is_checked,
            DryTrack(is_checked) => self.settings.dry_track = is_checked,
            DigitalOutLevel(val) => {
                if !self
                    .settings
                    .digital_out_level
                    .set(ui_to_jstation_normal(val))
                    .has_changed()
                {
                    return None;
                }

                self.digital_out_level_str = format!("{:02}", self.settings.digital_out_level);

                return Some((self.on_change)(Event::DigitalOutLevel(
                    self.settings.digital_out_level,
                )));
            }
            GlobalCabinet(is_checked) => self.settings.global_cabinet = is_checked,
            MidiMerge(is_checked) => self.settings.midi_merge = is_checked,
            MidiChannel(val) => {
                if !self
                    .settings
                    .midi_channel
                    .set(ui_to_jstation_normal(val))
                    .has_changed()
                {
                    return None;
                }

                self.midi_channel_str = format!("{:02}", self.settings.midi_channel);

                self.midi_channel_changed = true;

                // Don't propagate just yet, wait for `MidiChannelReleased`.
                return None;
            }
            MidiChannelReleased => {
                if !self.midi_channel_changed {
                    return None;
                }

                // Propagating change => reset flag
                self.midi_channel_changed = false;
            }
        }

        Some((self.on_change)(Event::UtilitySettings(self.settings)))
    }

    fn view(&self, _state: &Self::State) -> Element<PrivEvent> {
        use PrivEvent::*;

        let content: Element<_> = row![
            column![
                checkbox("Stereo", self.settings.stereo_mono, Stereo),
                checkbox("Dry Track", self.settings.dry_track, DryTrack),
            ]
            .spacing(10)
            .padding(5),
            column![
                checkbox(
                    "Global Cabinet",
                    self.settings.global_cabinet,
                    GlobalCabinet,
                ),
                checkbox("Midi Merge", self.settings.midi_merge, MidiMerge),
            ]
            .spacing(10)
            .padding(5),
            column![
                text("Midi Channel"),
                row![
                    Knob::new(
                        jstation_to_ui_param(self.settings.midi_channel),
                        MidiChannel,
                    )
                    .on_release(|| Some(MidiChannelReleased))
                    .size(KNOB_SIZE),
                    text(&self.midi_channel_str),
                ]
                .spacing(5)
                .align_items(Alignment::Center),
            ]
            .align_items(Alignment::Center),
            column![
                text("Digital Out Level"),
                row![
                    Knob::new(
                        jstation_to_ui_param(self.settings.digital_out_level),
                        DigitalOutLevel,
                    )
                    .size(KNOB_SIZE),
                    text(&self.digital_out_level_str),
                ]
                .spacing(5)
                .align_items(Alignment::Center),
            ]
            .align_items(Alignment::Center),
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

impl<'a, Message: 'a> From<Panel<'a, Message>> for Element<'a, Message, iced::Renderer> {
    fn from(panel: Panel<'a, Message>) -> Self {
        iced_lazy::component(panel)
    }
}
