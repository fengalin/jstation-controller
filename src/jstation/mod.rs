pub mod channel_voice;
pub use channel_voice::ChannelVoice;

pub mod data;
pub use data::{dsp, CCParameter, Program, ProgramData, ProgramId, ProgramNb, ProgramsBank};

mod error;
pub use error::Error;

mod interface;
pub use interface::{Interface, Listener};

mod sysex;
pub use sysex::{
    take_split_bytes_bool, take_split_bytes_chan, take_split_bytes_len, take_split_bytes_u16,
    take_split_bytes_u8, take_u8, BufferBuilder,
};

pub mod procedure;
pub use procedure::{Procedure, ProcedureBuilder};

use nom::{branch::alt, IResult};
use std::sync::Arc;

use crate::midi;

#[derive(Debug, Clone)]
pub enum Message {
    ChannelVoice(ChannelVoice),
    SysEx(Arc<sysex::Message>),
}

impl From<ChannelVoice> for Message {
    fn from(cv: ChannelVoice) -> Self {
        Message::ChannelVoice(cv)
    }
}

impl From<sysex::Message> for Message {
    fn from(msg: sysex::Message) -> Self {
        Message::SysEx(msg.into())
    }
}

fn parse_midi_channel_voice(input: &[u8]) -> IResult<&[u8], Message> {
    let (i, cv) = midi::channel_voice::parse(input)?;

    let cv = ChannelVoice::try_from(cv).map_err(|err| {
        use nom::error::{self, Error};

        log::debug!("{err}");
        nom::Err::Error(Error::new(input, error::ErrorKind::NoneOf))
    })?;

    Ok((i, cv.into()))
}

fn parse_sysex(i: &[u8]) -> IResult<&[u8], Message> {
    sysex::parse(i).map(|(i, msg)| (i, msg.into()))
}

pub fn parse_raw_midi_msg(i: &[u8]) -> IResult<&[u8], Message> {
    alt((parse_sysex, parse_midi_channel_voice))(i)
}

pub mod split_bytes {
    use crate::midi::Channel;

    pub fn from_bool(val: bool) -> [u8; 2] {
        [0, val.into()]
    }

    #[track_caller]
    pub fn to_bool(sb: &[u8]) -> bool {
        sb[1] != 0
    }

    pub fn from_u8(val: u8) -> [u8; 2] {
        [val >> 7, val & 0x7f]
    }

    #[track_caller]
    pub fn to_u8(sb: &[u8]) -> u8 {
        (sb[0] << 7) + sb[1]
    }

    pub fn from_chan(chan: Channel) -> [u8; 2] {
        from_u8(chan.into())
    }

    pub fn from_u16(val: u16) -> [u8; 4] {
        let lsb = (val & 0xff) as u8;
        let msb = (val >> 8) as u8;

        [lsb >> 7, lsb & 0x7f, msb >> 7, msb & 0x7f]
    }

    #[track_caller]
    pub fn to_u16(sb: &[u8]) -> u16 {
        (((sb[0] << 7) + sb[1]) as u16) + ((sb[2] as u16) << 15) + ((sb[3] as u16) << 8)
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn to_u8() {
            assert_eq!(super::to_u8(&[0, 0]), 0);
            assert_eq!(super::to_u8(&[0, 1]), 1);
            assert_eq!(super::to_u8(&[0, 8]), 8);
            assert_eq!(super::to_u8(&[1, 8]), 0x88);
        }

        #[test]
        fn from_u8() {
            assert_eq!(super::from_u8(0), [0, 0]);
            assert_eq!(super::from_u8(1), [0, 1]);
            assert_eq!(super::from_u8(8), [0, 8]);
            assert_eq!(super::from_u8(0x88), [1, 8]);
        }
    }
}
