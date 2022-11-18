use iced::Subscription;
use iced_native::subscription;

use std::{cell::Cell, sync::Arc};

use crate::{
    jstation::{procedure, sysex, Procedure, ProcedureBuilder},
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

#[derive(Debug, Clone)]
pub enum Message {
    Procedure(Arc<Procedure>),
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

    pub fn set_channels(
        &mut self,
        cc_chan: midi::Channel,
        sysex_chan: midi::Channel,
    ) -> Result<(), Error> {
        self.cc_chan = cc_chan;
        self.sysex_chan = sysex_chan;

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
                subscription::unfold(DEVICE_SUBSCRIPTION, Some(iface), iface_subscription)
            }
            Device => {
                // Keep going with the subscription started in the Midi(_) stage
                self.stage.set(Device);
                subscription::unfold(DEVICE_SUBSCRIPTION, None, iface_subscription)
            }
        }
    }
}

async fn iface_subscription(
    mut iface: Option<AsyncInterface>,
) -> (Option<Result<Message, Error>>, Option<AsyncInterface>) {
    let res = {
        let iface = iface.as_mut().expect("Wrong state");
        iface.listen().await
    };

    (Some(res), iface)
}

struct AsyncInterface {
    cc_chan: midi::Channel,
    sysex_chan: midi::Channel,
    msg_rx: flume::Receiver<Vec<u8>>,
    _midi_in: midir::MidiInputConnection<flume::Sender<Vec<u8>>>,
}

impl AsyncInterface {
    async fn listen(&mut self) -> Result<Message, Error> {
        loop {
            let sysex_msg = self.receive().await?;

            if let Procedure::WhoAmIResp(resp) = sysex_msg.proc {
                // FIXME is this the one to listen to?
                self.cc_chan = resp.transmit_chan;
                self.sysex_chan = resp.sysex_chan;

                log::debug!(
                    "Found device. Using cc rx {:?} tx {:?} & sysex {:?}",
                    resp.receive_chan,
                    resp.transmit_chan,
                    resp.sysex_chan,
                );

                return Ok(Message::Procedure(sysex_msg.proc.into()));
            }

            if sysex_msg.chan == self.sysex_chan {
                log::debug!("Received {:?}", sysex_msg.proc);

                return Ok(Message::Procedure(sysex_msg.proc.into()));
            } else {
                log::debug!(
                    "Ignoring sysex on {:?}:  {:?}",
                    sysex_msg.chan,
                    sysex_msg.proc
                );
            }
        }
    }

    // FIXME also receive CC
    async fn receive(&mut self) -> Result<sysex::Message, Error> {
        // FIXME need to timeout if we decide to scan ports for device
        let midi_msg = self
            .msg_rx
            .recv_async()
            .await
            .map_err(|_| Error::BokenConnection)?;

        sysex::parse(&midi_msg)
            .map(|(_, proc)| proc)
            .map_err(|err| {
                // FIXME could distinguish between unknow Proc and parsing error
                log::error!("{}", err.to_string());

                Error::Parse
            })
    }
}
