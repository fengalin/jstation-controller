use nom::IResult;

use crate::{
    jstation::{
        data::RawValue, take_split_bytes_bool, take_split_bytes_chan, take_split_bytes_len,
        take_split_bytes_u8, BufferBuilder, ProcedureBuilder, ProcedureId,
    },
    midi,
};

#[derive(Debug)]
pub struct UtilitySettingsReq;

impl ProcedureId for UtilitySettingsReq {
    const ID: u8 = 0x11;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for UtilitySettingsReq {}

impl UtilitySettingsReq {
    pub fn parse<'i>(input: &'i [u8], _checksum: &mut u8) -> IResult<&'i [u8], UtilitySettingsReq> {
        Ok((input, UtilitySettingsReq))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UtilitySettingsResp {
    pub stereo_mono: bool,
    pub dry_track: bool,
    pub digital_out_level: RawValue,
    pub global_cabinet: bool,
    pub midi_merge: bool,
    pub midi_channel: midi::Channel,
}

impl ProcedureId for UtilitySettingsResp {
    const ID: u8 = 0x12;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for UtilitySettingsResp {
    fn push_variable_size_data(&self, buffer: &mut BufferBuilder) {
        let buf = [
            self.stereo_mono.into(),
            self.dry_track.into(),
            self.digital_out_level.as_u8(),
            self.global_cabinet.into(),
            self.midi_merge.into(),
            self.midi_channel.as_u8(),
        ];

        buffer.push_variable_size_data(buf.into_iter());
    }
}

impl UtilitySettingsResp {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], UtilitySettingsResp> {
        let (i, _) = take_split_bytes_len(input, checksum, 6)?;

        let (i, stereo_mono) = take_split_bytes_bool(i, checksum)?;
        let (i, dry_track) = take_split_bytes_bool(i, checksum)?;
        let (i, digital_out_level) = take_split_bytes_u8(i, checksum)?;
        let (i, global_cabinet) = take_split_bytes_bool(i, checksum)?;
        let (i, midi_merge) = take_split_bytes_bool(i, checksum)?;
        let (i, midi_channel) = take_split_bytes_chan(i, checksum)?;

        Ok((
            i,
            UtilitySettingsResp {
                stereo_mono,
                dry_track,
                digital_out_level: RawValue::from(digital_out_level),
                global_cabinet,
                midi_merge,
                midi_channel,
            },
        ))
    }
}
