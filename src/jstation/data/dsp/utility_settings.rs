use crate::jstation::procedure::UtilitySettingsResp;
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
    #[const_range(min = 1, max = 15, display_raw)]
    pub midi_channel: MidiChannel,
}

impl TryFrom<UtilitySettingsResp> for UtilitySettings {
    type Error = crate::jstation::Error;

    fn try_from(proc: UtilitySettingsResp) -> Result<Self, Self::Error> {
        use crate::jstation::data::ConstRangeParameter;

        Ok(UtilitySettings {
            stereo_mono: proc.stereo_mono,
            dry_track: proc.dry_track,
            digital_out_level: DigitalOutLevel::try_from_raw(proc.digital_out_level)?,
            global_cabinet: proc.global_cabinet,
            midi_merge: proc.midi_merge,
            midi_channel: MidiChannel::try_from_raw(proc.midi_channel.into())?,
        })
    }
}

impl TryFrom<&UtilitySettingsResp> for UtilitySettings {
    type Error = crate::jstation::Error;

    fn try_from(proc: &UtilitySettingsResp) -> Result<Self, Self::Error> {
        UtilitySettings::try_from(*proc)
    }
}
