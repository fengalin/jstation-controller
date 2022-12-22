use nom::{
    bytes::complete::take,
    error::{self, Error},
    IResult,
};

pub mod cc;
pub use cc::{CCNumber, CCValue, CC};

use crate::midi;

#[derive(Copy, Clone, Debug)]
pub struct ChannelVoice {
    pub chan: midi::Channel,
    pub msg: Message,
}

#[derive(Copy, Clone, Debug)]
pub enum Message {
    CC(CC),
    ProgramChange(u8),
}

pub fn parse(input: &[u8]) -> IResult<&[u8], ChannelVoice> {
    let (i, tag_chan) = take(1usize)(input)?;
    let tag_chan = midi::TagChannel::from(tag_chan[0]);

    let (i, msg) = match tag_chan.tag {
        CC::TAG => cc::parse(i).map(|(i, msg)| (i, Message::CC(msg)))?,
        midi::Tag(0xc0) => {
            // ProgramChange
            let (i, prog_id) = take(1usize)(i)?;
            (i, Message::ProgramChange(prog_id[0]))
        }
        other => {
            log::debug!(
                "Unknown Midi ChannelVoice tag with id: 0x{:02x}",
                other.as_u8(),
            );
            return Err(nom::Err::Error(Error::new(input, error::ErrorKind::NoneOf)));
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
