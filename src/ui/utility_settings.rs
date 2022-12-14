use iced::{
    widget::{checkbox, column, row, text},
    Alignment, Element, Length,
};
use iced_audio::Knob;
use iced_lazy::{self, Component};

use crate::{
    jstation::data::{
        dsp::{utility_settings, UtilitySettings},
        DiscreteParameter,
    },
    ui::{to_jstation_normal, to_ui_param},
};

const KNOB_SIZE: Length = Length::Units(35);

#[derive(Debug, Clone)]
pub enum Event {
    UtilitySettings(UtilitySettings),
    DigitalOutLevel(utility_settings::DigitalOutLevel),
}

#[derive(Debug)]
pub enum PrivEvent {
    Stereo(bool),
    DryTrack(bool),
    DigitalOutLevel(utility_settings::DigitalOutLevel),
    GlobalCabinet(bool),
    MidiMerge(bool),
    MidiChannel(utility_settings::MidiChannel),
    MidiChannelReleased,
}

pub struct Panel<'a, Message> {
    settings: UtilitySettings,
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
            midi_channel_changed: false,
            on_change: Box::new(on_change),
        }
    }
}

impl<'a, Message> Component<Message, iced::Renderer> for Panel<'a, Message> {
    type State = ();
    type Event = PrivEvent;

    fn update(&mut self, _state: &mut Self::State, event: PrivEvent) -> Option<Message> {
        use PrivEvent::*;
        match event {
            Stereo(is_checked) => self.settings.stereo_mono = is_checked,
            DryTrack(is_checked) => self.settings.dry_track = is_checked,
            DigitalOutLevel(digital_out_level) => {
                return self
                    .settings
                    .digital_out_level
                    .set(digital_out_level)
                    .map(|new_dol| (self.on_change)(Event::DigitalOutLevel(new_dol)));
            }
            GlobalCabinet(is_checked) => self.settings.global_cabinet = is_checked,
            MidiMerge(is_checked) => self.settings.midi_merge = is_checked,
            MidiChannel(chan) => {
                self.settings.midi_channel.set(chan)?;

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
                text(utility_settings::MidiChannel::NAME),
                row![
                    Knob::new(to_ui_param(self.settings.midi_channel), |normal| {
                        MidiChannel(utility_settings::MidiChannel::from_snapped(
                            to_jstation_normal(normal),
                        ))
                    })
                    .on_release(|| Some(MidiChannelReleased))
                    .size(KNOB_SIZE),
                    text(format!("{:02}", self.settings.midi_channel)),
                ]
                .spacing(5)
                .align_items(Alignment::Center),
            ]
            .align_items(Alignment::Center),
            column![
                text(utility_settings::DigitalOutLevel::NAME),
                row![
                    Knob::new(to_ui_param(self.settings.digital_out_level), |normal| {
                        DigitalOutLevel(utility_settings::DigitalOutLevel::from_snapped(
                            to_jstation_normal(normal),
                        ))
                    })
                    .size(KNOB_SIZE),
                    text(format!("{:02}", self.settings.digital_out_level)),
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
