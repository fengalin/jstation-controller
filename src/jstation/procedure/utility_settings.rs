use nom::IResult;
use smallvec::SmallVec;

use crate::{
    jstation::{split_bytes, take_split_bytes_bool, take_split_bytes_chan, take_split_bytes_len, take_split_bytes_u8, BufferBuilder, ProcedureBuilder},
    midi,
};

#[derive(Debug)]
pub struct UtilitySettingsReq;

impl ProcedureBuilder for UtilitySettingsReq {
    const ID: u8 = 0x11;
    const VERSION: u8 = 1;
}

impl UtilitySettingsReq {
    pub fn parse<'i>(i: &'i [u8], _checksum: &mut u8) -> IResult<&'i [u8], UtilitySettingsReq> {
        Ok((i, UtilitySettingsReq))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UtilitySettingsResp {
    pub stereo_mono: bool,
    pub dry_track: bool,
    pub digital_out_level: u8,
    pub global_cabinet: bool,
    pub midi_merge: bool,
    pub midi_channel: midi::Channel,
}

impl ProcedureBuilder for UtilitySettingsResp {
    const ID: u8 = 0x12;
    const VERSION: u8 = 1;

    fn push_variable_size_data(&self, buffer: &mut BufferBuilder) {
        let mut buf = SmallVec::<[u8; 2 * 6]>::new();
        buf.extend_from_slice(&split_bytes::from_bool(self.stereo_mono));
        buf.extend_from_slice(&split_bytes::from_bool(self.dry_track));
        buf.extend_from_slice(&split_bytes::from_u8(self.digital_out_level));
        buf.extend_from_slice(&split_bytes::from_bool(self.global_cabinet));
        buf.extend_from_slice(&split_bytes::from_bool(self.midi_merge));
        buf.extend_from_slice(&split_bytes::from_chan(self.midi_channel));

        buffer.push_variable_size_data(buf.into_iter());
    }
}

impl UtilitySettingsResp {
    pub fn parse<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], UtilitySettingsResp> {
        let (i, _) = take_split_bytes_len(i, checksum, 6)?;

        let (i, stereo_mono) = take_split_bytes_bool(i, checksum)?;
        let (i, dry_track) = take_split_bytes_bool(i, checksum)?;
        let (i, digital_out_level) = take_split_bytes_u8(i, checksum)?;
        let (i, global_cabinet) = take_split_bytes_bool(i, checksum)?;
        let (i, midi_merge) = take_split_bytes_bool(i, checksum)?;
        let (i, midi_channel) = take_split_bytes_chan(i, checksum)?;

        Ok((i, UtilitySettingsResp {
            stereo_mono,
            dry_track,
            digital_out_level,
            global_cabinet,
            midi_merge,
            midi_channel,
        }))
    }
}
