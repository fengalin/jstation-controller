use crate::{jstation::ProgramId, midi};

#[derive(Copy, Clone, Debug)]
pub struct ChannelVoice {
    pub chan: midi::Channel,
    pub msg: Message,
}

#[derive(Copy, Clone, Debug)]
pub enum Message {
    CC(midi::CC),
    ProgramChange(ProgramId),
}

impl From<midi::ChannelVoice> for ChannelVoice {
    fn from(cv: midi::ChannelVoice) -> Self {
        use midi::channel_voice::Message::*;
        let msg = match cv.msg {
            CC(cc) => Message::CC(cc),
            ProgramChange(midi_prog_nb) => Message::ProgramChange(ProgramId::from(midi_prog_nb)),
        };

        ChannelVoice { chan: cv.chan, msg }
    }
}
