use iced::{
    widget::{checkbox, column, row, text},
    Alignment, Element, Length,
};
use iced_audio::{IntRange, Knob, Normal, NormalParam};
use iced_lazy::{self, Component};
use once_cell::sync::Lazy;

use crate::{jstation::procedure::UtilitySettingsResp, midi};

const DEFAULT_DIGITAL_OUT_LEVEL: i32 = 12;
static DIGITAL_OUT_LEVEL_RANGE: Lazy<IntRange> = Lazy::new(|| IntRange::new(0, 24));

const DEFAULT_MIDI_CHANNEL: i32 = 0;
static MIDI_CHANNEL_RANGE: Lazy<IntRange> = Lazy::new(|| IntRange::new(0, 15));

const KNOB_SIZE: Length = Length::Units(35);

#[derive(Clone, Copy, Debug, Default)]
pub struct Settings {
    stereo_mono: bool,
    dry_track: bool,
    digital_out_level: NormalParam,
    global_cabinet: bool,
    midi_merge: bool,
    midi_channel: NormalParam,
}

impl From<UtilitySettingsResp> for Settings {
    fn from(val: UtilitySettingsResp) -> Self {
        Settings {
            stereo_mono: val.stereo_mono,
            dry_track: val.dry_track,
            digital_out_level: DIGITAL_OUT_LEVEL_RANGE
                .normal_param(val.digital_out_level as i32, DEFAULT_DIGITAL_OUT_LEVEL),
            global_cabinet: val.global_cabinet,
            midi_merge: val.midi_merge,
            midi_channel: MIDI_CHANNEL_RANGE
                .normal_param(u8::from(val.midi_channel) as i32, DEFAULT_MIDI_CHANNEL),
        }
    }
}

impl From<Settings> for UtilitySettingsResp {
    fn from(val: Settings) -> Self {
        UtilitySettingsResp {
            stereo_mono: val.stereo_mono,
            dry_track: val.dry_track,
            digital_out_level: DIGITAL_OUT_LEVEL_RANGE.unmap_to_value(val.digital_out_level.value)
                as u8,
            global_cabinet: val.global_cabinet,
            midi_merge: val.midi_merge,
            midi_channel: (MIDI_CHANNEL_RANGE.unmap_to_value(val.midi_channel.value) as u8).into(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    UtilitySettings(UtilitySettingsResp),
    DigitalOutLevel(u8),
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
    settings: Settings,
    digital_out_level_str: String,
    midi_channel_str: String,
    midi_channel_changed: bool,
    on_change: Box<dyn 'a + Fn(Event) -> Message>,
}

impl<'a, Message> Panel<'a, Message> {
    pub fn new<F>(settings: UtilitySettingsResp, on_change: F) -> Self
    where
        F: 'a + Fn(Event) -> Message,
    {
        Self {
            settings: settings.into(),
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

    fn update(&mut self, _state: &mut Self::State, mut event: PrivEvent) -> Option<Message> {
        use PrivEvent::*;
        match event {
            Stereo(is_checked) => self.settings.stereo_mono = is_checked,
            DryTrack(is_checked) => self.settings.dry_track = is_checked,
            DigitalOutLevel(val) => {
                let val = DIGITAL_OUT_LEVEL_RANGE.snapped(val);
                if self.settings.digital_out_level.value == val {
                    return None;
                }

                self.settings.digital_out_level.value = val;
                let val = DIGITAL_OUT_LEVEL_RANGE.unmap_to_value(val) as u8;
                self.digital_out_level_str = format!("{val:02}");

                return Some((self.on_change)(Event::DigitalOutLevel(val)));
            }
            GlobalCabinet(is_checked) => self.settings.global_cabinet = is_checked,
            MidiMerge(is_checked) => self.settings.midi_merge = is_checked,
            MidiChannel(ref mut val) => {
                *val = MIDI_CHANNEL_RANGE.snapped(*val);
                if self.settings.midi_channel.value == *val {
                    return None;
                }

                self.settings.midi_channel.value = *val;
                let chan = midi::Channel::from(MIDI_CHANNEL_RANGE.unmap_to_value(*val) as u8);
                self.midi_channel_str = format!("{chan:02}");

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

        Some((self.on_change)(Event::UtilitySettings(
            self.settings.into(),
        )))
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
                    Knob::new(self.settings.midi_channel, MidiChannel)
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
                    Knob::new(self.settings.digital_out_level, DigitalOutLevel).size(KNOB_SIZE),
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
