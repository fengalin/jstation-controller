use nom::{bytes::complete::take, IResult};

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
    ProgramChange(ProgramNumber),
}

pub fn parse(input: &[u8]) -> IResult<&[u8], ChannelVoice> {
    let (i, tag_chan) = take(1usize)(input)?;
    let tag_chan = midi::TagChannel::from(tag_chan[0]);

    let (i, msg) = match tag_chan.tag {
        CC::TAG => CC::parse(i).map(|(i, msg)| (i, Message::CC(msg)))?,
        ProgramChange::TAG => {
            ProgramNumber::parse(i).map(|(i, msg)| (i, Message::ProgramChange(msg)))?
        }
        other => {
            log::debug!(
                "Unknown Midi ChannelVoice tag with id: 0x{:02x}",
                other.as_u8(),
            );
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::NoneOf,
            )));
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

pub struct ProgramChange;

impl ProgramChange {
    pub const TAG: midi::Tag = midi::Tag(0xc0);

    pub fn build_for(prog_nb: ProgramNumber, chan: midi::Channel) -> [u8; 2] {
        let tag_chan = midi::TagChannel {
            tag: Self::TAG,
            chan,
        };

        [tag_chan.into(), prog_nb.0]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ProgramNumber(u8);

impl ProgramNumber {
    pub fn parse(i: &[u8]) -> IResult<&[u8], ProgramNumber> {
        let (i, prog_id) = take(1usize)(i)?;

        Ok((i, ProgramNumber(prog_id[0])))
    }
}

impl From<u8> for ProgramNumber {
    fn from(prog: u8) -> Self {
        ProgramNumber(prog)
    }
}

impl From<ProgramNumber> for u8 {
    fn from(prog: ProgramNumber) -> Self {
        prog.0
    }
}
