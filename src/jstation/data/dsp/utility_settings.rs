use crate::{
    jstation::{
        data::{DiscreteParameter, Normal, RawValue},
        procedure::UtilitySettingsResp,
    },
    midi::CCNumber,
};

discrete_parameter!(
    #[derive(Display)]
    DigitalOutLevel {
        const DEFAULT = Normal::MIN,
        const MAX_RAW = RawValue::new(24),
        const CC_NB = CCNumber::new(14),
    }
);

discrete_parameter!(
    #[derive(Display)]
    MidiChannel {
        const DEFAULT = Normal::MIN,
        const MIN_RAW = RawValue::new(1),
        const MAX_RAW = RawValue::new(15),
    }
);

#[derive(Clone, Copy, Debug, Default)]
pub struct UtilitySettings {
    pub stereo_mono: bool,
    pub dry_track: bool,
    pub digital_out_level: DigitalOutLevel,
    pub global_cabinet: bool,
    pub midi_merge: bool,
    pub midi_channel: MidiChannel,
}

impl TryFrom<UtilitySettingsResp> for UtilitySettings {
    type Error = crate::jstation::Error;

    fn try_from(proc: UtilitySettingsResp) -> Result<Self, Self::Error> {
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
