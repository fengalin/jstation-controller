use crate::{jstation::ProgramNumber, midi};

#[derive(Copy, Clone, Debug)]
pub struct ChannelVoice {
    pub chan: midi::Channel,
    pub msg: Message,
}

#[derive(Copy, Clone, Debug)]
pub enum Message {
    CC(midi::CC),
    ProgramChange(ProgramNumber),
}

impl From<midi::ChannelVoice> for ChannelVoice {
    fn from(cv: midi::ChannelVoice) -> Self {
        use midi::channel_voice::Message::*;
        let msg = match cv.msg {
            CC(cc) => Message::CC(cc),
            ProgramChange(prog_nb) => Message::ProgramChange(ProgramNumber::from(prog_nb)),
        };

        ChannelVoice { chan: cv.chan, msg }
    }
}
