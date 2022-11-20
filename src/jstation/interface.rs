use iced::Subscription;
use iced_native::subscription;

use std::{cell::Cell, sync::Arc};

use crate::{
    jstation::{parse_raw_midi_msg, procedure, Message, Procedure, ProcedureBuilder},
    midi,
};

const DEVICE_SUBSCRIPTION: &str = "device";

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Error Parsing MIDI message")]
    Parse,
    #[error("No MIDI connection")]
    MidiNotConnected,
    #[error("An error occured sending a MIDI message")]
    MidiSend,
    #[error("Broken MIDI connection")]
    BokenConnection,
}

#[derive(Default)]
enum ConnectionStage {
    #[default]
    Disconnected,
    Midi(AsyncInterface),
    Device,
}

pub struct Interface {
    pub ins: midi::PortsIn,
    pub outs: midi::PortsOut,
    // Need iterior mutability so as to change stage in subscription()
    stage: Cell<ConnectionStage>,
    midi_out: Option<midir::MidiOutputConnection>,
    cc_chan: midi::Channel,
    sysex_chan: midi::Channel,
}

// FIXME disconnect on shutdown
impl Interface {
    pub fn new(app_name: Arc<str>) -> Self {
        Interface {
            ins: midi::PortsIn::new(app_name.clone()),
            outs: midi::PortsOut::new(app_name),
            stage: Default::default(),
            midi_out: None,
            // FIXME check which value should be used initially
            cc_chan: midi::Channel::ALL,
            sysex_chan: midi::Channel::ALL,
        }
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        self.ins.refresh()?;
        self.outs.refresh()?;

        Ok(())
    }

    pub fn have_who_am_i_resp(&mut self, resp: &procedure::WhoAmIResp) -> Result<(), Error> {
        // FIXME check that this is the right channel to send cc
        self.cc_chan = resp.receive_chan;
        self.sysex_chan = resp.sysex_chan;

        log::debug!("Sending UtilitySettingsReq");
        let res = self.send_sysex(procedure::UtilitySettingsReq);
        if res.is_err() {
            // FIXME should probably assume connection to be broken
        }

        res
    }

    fn send_sysex(&mut self, proc: impl ProcedureBuilder) -> Result<(), Error> {
        self.send(&proc.build_for(self.sysex_chan))
    }

    fn send(&mut self, msg: &[u8]) -> Result<(), Error> {
        self.midi_out
            .as_mut()
            .ok_or(Error::MidiNotConnected)?
            .send(msg)
            .map_err(|_| Error::MidiSend)
    }

    pub fn connect(&mut self, port_in: Arc<str>, port_out: Arc<str>) -> anyhow::Result<()> {
        let mut midi_out = self.outs.connect(port_out)?;

        let (msg_tx, msg_rx) = flume::bounded(10);
        let midi_in = self.ins.connect(port_in, msg_tx, |_ts, msg, msg_tx| {
            let _ = msg_tx.send(msg.to_owned());
        })?;

        log::debug!("Sending WhoAmIReq");
        midi_out
            .send(&procedure::WhoAmIReq::default().build_for(midi::Channel::ALL))
            .map_err(|_| Error::MidiSend)?;

        self.stage.set(ConnectionStage::Midi(AsyncInterface {
            cc_chan: midi::Channel::default(),
            sysex_chan: midi::Channel::default(),
            msg_rx,
            _midi_in: midi_in,
        }));
        self.midi_out = Some(midi_out);

        Ok(())
    }

    pub fn subscription(&self) -> Subscription<Result<Message, Error>> {
        use ConnectionStage::*;
        match self.stage.take() {
            Disconnected => Subscription::none(),
            Midi(iface) => {
                self.stage.set(Device);
                // FIXME start with a dedicate subscription that purges any
                // pending messages and which is able to timeout on whoamiresp
                subscription::unfold(DEVICE_SUBSCRIPTION, Some(iface), iface_listen_subscription)
            }
            Device => {
                // Keep going with the subscription started in the Midi(_) stage
                self.stage.set(Device);
                subscription::unfold(DEVICE_SUBSCRIPTION, None, iface_listen_subscription)
            }
        }
    }
}

async fn iface_listen_subscription(
    mut iface: Option<AsyncInterface>,
) -> (Option<Result<Message, Error>>, Option<AsyncInterface>) {
    if let Some(iface_mut) = iface.as_mut() {
        let res = iface_mut.listen().await;
        (Some(res), iface)
    } else {
        (None, None)
    }
}

struct AsyncInterface {
    cc_chan: midi::Channel,
    sysex_chan: midi::Channel,
    msg_rx: flume::Receiver<Vec<u8>>,
    _midi_in: midir::MidiInputConnection<flume::Sender<Vec<u8>>>,
}

impl AsyncInterface {
    async fn listen(&mut self) -> Result<Message, Error> {
        use Message::*;

        loop {
            let msg = self.receive().await?;
            match &msg {
                ChannelVoice(cv) => {
                    if cv.chan == self.cc_chan {
                        log::debug!("Received {:?}", cv.msg);

                        return Ok(msg);
                    }

                    log::debug!("Ignoring channel voice on {:?}: {:?}", cv.chan, cv.msg);
                }
                SysEx(sysex) => {
                    if let Procedure::WhoAmIResp(resp) = sysex.proc {
                        // FIXME is this the one to listen to?
                        self.cc_chan = resp.transmit_chan;
                        self.sysex_chan = resp.sysex_chan;

                        log::debug!(
                            "Found device. Using cc rx {:?} tx {:?} & sysex {:?}",
                            resp.receive_chan,
                            resp.transmit_chan,
                            resp.sysex_chan,
                        );

                        return Ok(msg);
                    }

                    if sysex.chan == self.sysex_chan {
                        log::debug!("Received {:?}", sysex.proc);

                        return Ok(msg);
                    }

                    log::debug!("Ignoring sysex on {:?}: {:?}", sysex.chan, sysex.proc);
                }
            }
        }
    }

    async fn receive(&mut self) -> Result<Message, Error> {
        let midi_msg = self
            .msg_rx
            .recv_async()
            .await
            .map_err(|_| Error::BokenConnection)?;

        parse_raw_midi_msg(&midi_msg)
            .map(|(_, proc)| proc)
            .map_err(|err| {
                log::error!("{}", err.to_string());

                Error::Parse
            })
    }
}
