use crate::{
    jstation::{CCParameter, Error, Parameter, ProgramNumber},
    midi,
};

#[derive(Copy, Clone, Debug)]
pub struct ChannelVoice {
    pub chan: midi::Channel,
    pub msg: Message,
}

#[derive(Copy, Clone, Debug)]
pub enum Message {
    CC(Parameter),
    ProgramChange(ProgramNumber),
}

impl TryFrom<midi::ChannelVoice> for ChannelVoice {
    type Error = Error;

    fn try_from(cv: midi::ChannelVoice) -> Result<Self, Self::Error> {
        use midi::channel_voice::Message::*;
        let msg = match cv.msg {
            CC(cc) => Message::CC(Parameter::from_cc(cc).ok_or_else(|| {
                let err = Error::CCNumberUnknown(cc.nb.into());
                log::warn!("{err}");

                err
            })?),
            ProgramChange(prog_nb) => Message::ProgramChange(ProgramNumber::from(prog_nb)),
        };

        Ok(ChannelVoice { chan: cv.chan, msg })
    }
}
