use nom::{
    bytes::complete::take,
    error::{self, Error},
    IResult,
};

pub mod cc;
pub use cc::CC;

use crate::midi;

#[derive(Copy, Clone, Debug)]
pub struct ChannelVoice {
    pub chan: midi::Channel,
    pub msg: Message,
}

#[derive(Copy, Clone, Debug)]
pub enum Message {
    CC(CC),
    // FIXME check range / use specific type
    ProgramChange(u8),
}

pub fn parse(i: &[u8]) -> IResult<&[u8], ChannelVoice> {
    let (i, tag_chan) = take(1usize)(i)?;
    let tag_chan = midi::TagChannel::from(tag_chan[0]);

    let (i, msg) = match tag_chan.tag {
        CC::TAG => cc::parse(i).map(|(i, msg)| (i, Message::CC(msg)))?,
        midi::Tag(0xc0) => {
            // ProgramChange
            let (i, prog_id) = take(1usize)(i)?;
            (i, Message::ProgramChange(prog_id[0]))
        }
        other => {
            log::warn!(
                "Unknown Midi ChannelVoice tag with id: 0x{:02x}",
                u8::from(other),
            );
            return Err(nom::Err::Failure(Error::new(i, error::ErrorKind::NoneOf)));
        }
    };

    Ok((
        i,
        ChannelVoice {
            chan: tag_chan.chan,
            msg,
        },
    ))
}
