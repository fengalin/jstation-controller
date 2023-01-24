use std::fmt;

use crate::jstation::{data::RawValue, prelude::*, procedure::UtilitySettingsResp, Error};
use crate::midi;
use jstation_derive::ParameterSetter;

// FIXME might be easier not to auto implement ParameterSetter
#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct UtilitySettings {
    pub stereo_mono: bool,
    pub dry_track: bool,
    #[const_range(max = 24, cc_nb = 14, display_raw)]
    pub digital_out_level: DigitalOutLevel,
    pub global_cabinet: bool,
    pub midi_merge: bool,
    #[const_range(max = 16)]
    pub midi_channel: MidiChannel,
}

impl MidiChannel {
    const ALL: MidiChannel = MidiChannel(RawValue::new(16));
}

impl fmt::Display for MidiChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == MidiChannel::ALL {
            return f.write_str("All");
        }

        fmt::Display::fmt(&(self.raw_value().as_u8() + 1), f)
    }
}

impl From<MidiChannel> for midi::Channel {
    fn from(chan: MidiChannel) -> Self {
        if chan == MidiChannel::ALL {
            return midi::Channel::ALL;
        }

        midi::Channel::from(chan.raw_value().as_u8())
    }
}

impl TryFrom<midi::Channel> for MidiChannel {
    type Error = Error;

    fn try_from(chan: midi::Channel) -> Result<Self, Self::Error> {
        if chan == midi::Channel::ALL {
            return Ok(MidiChannel::ALL);
        }

        let chan = chan.as_u8();
        if chan > 15 {
            return Err(Error::MidiChannelOutOfRange(chan));
        }

        Ok(MidiChannel(chan.into()))
    }
}

impl TryFrom<UtilitySettingsResp> for UtilitySettings {
    type Error = Error;

    fn try_from(proc: UtilitySettingsResp) -> Result<Self, Self::Error> {
        Ok(UtilitySettings {
            stereo_mono: proc.stereo_mono,
            dry_track: proc.dry_track,
            digital_out_level: DigitalOutLevel::try_from_raw(proc.digital_out_level)?,
            global_cabinet: proc.global_cabinet,
            midi_merge: proc.midi_merge,
            midi_channel: MidiChannel::try_from(proc.midi_channel)?,
        })
    }
}

impl TryFrom<&UtilitySettingsResp> for UtilitySettings {
    type Error = Error;

    fn try_from(proc: &UtilitySettingsResp) -> Result<Self, Self::Error> {
        UtilitySettings::try_from(*proc)
    }
}

impl From<UtilitySettings> for UtilitySettingsResp {
    fn from(settings: UtilitySettings) -> Self {
        UtilitySettingsResp {
            stereo_mono: settings.stereo_mono,
            dry_track: settings.dry_track,
            digital_out_level: settings.digital_out_level.raw_value(),
            global_cabinet: settings.global_cabinet,
            midi_merge: settings.midi_merge,
            midi_channel: settings.midi_channel.into(),
        }
    }
}
