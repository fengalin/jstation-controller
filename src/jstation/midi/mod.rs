pub mod channel_voice;
pub use channel_voice::ChannelVoice;

pub mod split_bytes;

pub mod sysex;
pub use sysex::{
    take_split_bytes_bool, take_split_bytes_chan, take_split_bytes_len, take_split_bytes_u16,
    take_split_bytes_u8, take_u8, BufferBuilder,
};

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

    Ok((i, ChannelVoice::from(cv).into()))
}

fn parse_sysex(i: &[u8]) -> IResult<&[u8], Message> {
    sysex::parse(i).map(|(i, msg)| (i, msg.into()))
}

pub fn parse_raw_midi_msg(i: &[u8]) -> IResult<&[u8], Message> {
    alt((parse_sysex, parse_midi_channel_voice))(i)
}
