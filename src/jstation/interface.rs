use std::sync::Arc;

use crate::{
    jstation::{
        self, dsp, parse_raw_midi_msg, procedure, sysex, Error, Message, Procedure,
        ProcedureBuilder, Program,
    },
    midi,
};

use std::time::Duration;

const HANDSHAKE_TIMEOUT: Duration = Duration::from_millis(200);

pub struct Interface {
    pub ins: midi::PortsIn,
    pub outs: midi::PortsOut,
    midi_out: Option<midir::MidiOutputConnection>,
    cc_chan: midi::Channel,
    sysex_chan: midi::Channel,
    chan_tx: Option<flume::Sender<midi::Channel>>,
}

/// General Interface behaviour.
impl Interface {
    pub fn new(app_name: Arc<str>) -> Self {
        Interface {
            ins: midi::PortsIn::new(app_name.clone()),
            outs: midi::PortsOut::new(app_name),
            midi_out: None,
            cc_chan: midi::Channel::ALL,
            sysex_chan: midi::Channel::ALL,
            chan_tx: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.midi_out.is_some()
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        self.ins.refresh()?;
        self.outs.refresh()?;

        Ok(())
    }

    pub fn clear(&mut self) {
        self.ins.disconnect();

        self.outs.disconnect();
        if let Some(midi_out) = self.midi_out.take() {
            midi_out.close();
            self.chan_tx = None;
        }
    }

    pub fn change_chan(&mut self, chan: midi::Channel) {
        // Note: I couldn't find the expected behaviour when changing the channel to All
        // in Utility Settings on the J-Station.
        if let Some(chan_tx) = self.chan_tx.as_mut() {
            chan_tx.send(chan).expect("Broken chan channel");
            self.cc_chan = chan;
            self.sysex_chan = chan;
        }
    }

    pub fn have_who_am_i_resp(&mut self, resp: procedure::WhoAmIResp) -> Result<(), Error> {
        self.cc_chan = resp.receive_chan;
        self.sysex_chan = resp.sysex_chan;

        log::debug!("Sending UtilitySettingsReq");
        self.request_utility_settings()
    }

    pub fn request_utility_settings(&mut self) -> Result<(), Error> {
        self.send_sysex(procedure::UtilitySettingsReq)
            .map_err(|err| Error::with_context("Utility Settings req.", err))
    }

    pub fn bank_dump(&mut self) -> Result<(), Error> {
        self.send_sysex(procedure::BankDumpReq)
            .map_err(|err| Error::with_context("Bank Dump req.", err))
    }

    pub fn program_update_req(&mut self) -> Result<(), Error> {
        self.send_sysex(procedure::ProgramUpdateReq)
            .map_err(|err| Error::with_context("Program Update req.", err))
    }

    pub fn change_program(&mut self, id: impl Into<midi::ProgramNumber>) -> Result<(), Error> {
        self.send(&midi::ProgramChange::build_for(id.into(), self.cc_chan))
    }

    pub fn request_program(&mut self, id: jstation::ProgramId) -> Result<(), Error> {
        self.send_sysex(procedure::OneProgramReq { id })
            .map_err(|err| Error::with_context("Program req.", err))
    }

    pub fn reload_program(&mut self) -> Result<(), Error> {
        self.send_sysex(procedure::ReloadProgramReq)
            .map_err(|err| Error::with_context("Reload Program req.", err))
    }

    pub fn store_program(&mut self, prog: &Program) -> Result<(), Error> {
        // Note: gstation-edit also sends a ProgramUpdateResp, but the
        // result seems to be the same with only OneProgramResp.
        // Or is it needed to change program's name?

        // FIXME there's also StoreProgram procedure with id 0x21
        // And NotifyStore is described as: Store Terminated
        self.send_sysex(procedure::OneProgramResp::from(prog))
            .map_err(|err| Error::with_context("Store One Program resp.", err))
    }

    pub fn update_utility_settings(&mut self, settings: dsp::UtilitySettings) -> Result<(), Error> {
        let resp: procedure::UtilitySettingsResp = settings.into();
        self.send_sysex(resp)
    }

    fn send_sysex<'a>(&mut self, proc: impl 'a + ProcedureBuilder) -> Result<(), Error> {
        self.send(&proc.build_for(self.sysex_chan))
    }

    pub fn send_cc(&mut self, cc: midi::CC) -> Result<(), Error> {
        self.send(&cc.build_for(self.cc_chan))
    }

    fn send(&mut self, msg: &[u8]) -> Result<(), Error> {
        self.midi_out
            .as_mut()
            .ok_or(Error::MidiNotConnected)?
            .send(msg)
            .map_err(|_| Error::MidiSend)
    }

    fn start_handshake(&mut self, midi_out: &mut midir::MidiOutputConnection) -> Result<(), Error> {
        self.cc_chan = midi::Channel::ALL;
        self.sysex_chan = midi::Channel::ALL;

        log::debug!("Sending WhoAmIReq");
        midi_out
            .send(&procedure::WhoAmIReq::default().build_for(midi::Channel::ALL))
            .map_err(|_| Error::MidiSend)
    }

    pub fn connected_ports(&self) -> Option<(Arc<str>, Arc<str>)> {
        self.ins.cur().zip(self.outs.cur())
    }
}

impl Drop for Interface {
    fn drop(&mut self) {
        if let Some(midi_out) = self.midi_out.take() {
            midi_out.close();
        }
    }
}

impl midi::Scannable for Interface {
    type In = Listener;
    type Out = ();
    type Error = Error;

    fn ins(&self) -> &midi::PortsIn {
        &self.ins
    }

    fn outs(&self) -> &midi::PortsOut {
        &self.outs
    }

    fn connect(&mut self, port_in: Arc<str>, port_out: Arc<str>) -> Result<(Listener, ()), Error> {
        let mut midi_out = self.outs.connect(port_out)?;

        let (chan_tx, chan_rx) = flume::bounded(1);
        let listener = Listener::try_new(self, port_in, chan_rx)?;
        self.chan_tx = Some(chan_tx);

        self.start_handshake(&mut midi_out)?;

        self.midi_out = Some(midi_out);

        Ok((listener, ()))
    }

    fn connect_in(&mut self, port_name: Arc<str>) -> Result<Listener, Error> {
        let (chan_tx, chan_rx) = flume::bounded(1);
        let listener = Listener::try_new(self, port_name, chan_rx)?;
        self.chan_tx = Some(chan_tx);

        self.cc_chan = midi::Channel::ALL;
        self.sysex_chan = midi::Channel::ALL;

        Ok(listener)
    }

    fn connect_out(&mut self, port_name: Arc<str>) -> Result<(), Error> {
        let mut midi_out = self.outs.connect(port_name)?;

        if let Some(prev_midi_out) = self.midi_out.take() {
            prev_midi_out.close();
        }

        self.start_handshake(&mut midi_out)?;

        self.midi_out = Some(midi_out);

        Ok(())
    }
}

#[derive(Debug)]
enum ListenerState {
    AwaitingHandshake,
    FoundDevice,
}

pub struct Listener {
    state: ListenerState,
    cc_chan: midi::Channel,
    sysex_chan: midi::Channel,
    msg_rx: flume::Receiver<Vec<u8>>,
    midi_in: Option<midir::MidiInputConnection<flume::Sender<Vec<u8>>>>,
    chan_rx: flume::Receiver<midi::Channel>,
}

impl Listener {
    fn try_new(
        iface: &mut Interface,
        port_in: Arc<str>,
        chan_rx: flume::Receiver<midi::Channel>,
    ) -> Result<Self, Error> {
        let (msg_tx, msg_rx) = flume::bounded(10);
        let midi_in = iface.ins.connect(port_in, msg_tx, |_ts, msg, msg_tx| {
            let _ = msg_tx.send(msg.to_owned());
        })?;

        Ok(Listener {
            state: ListenerState::AwaitingHandshake,
            cc_chan: midi::Channel::default(),
            sysex_chan: midi::Channel::default(),
            msg_rx,
            midi_in: Some(midi_in),
            chan_rx,
        })
    }

    pub async fn listen(&mut self) -> Result<Message, Error> {
        use ListenerState::*;
        match self.state {
            FoundDevice => self.await_device_message().await,
            AwaitingHandshake => self.await_handshake().await,
        }
    }

    /// Awaits for a device message.
    ///
    /// Listen to incoming MIDI messages on current `midi_in` returning
    /// the message if it comed from the device found during last handshake.
    async fn await_device_message(&mut self) -> Result<Message, Error> {
        loop {
            let msg = self.receive().await?;
            use Message::*;
            match &msg {
                ChannelVoice(cv) => {
                    if cv.chan == self.cc_chan {
                        log::trace!("Received {:?}", cv.msg);

                        return Ok(msg);
                    }

                    log::trace!("Ignoring channel voice on {}: {:?}", cv.chan, cv.msg);
                }
                SysEx(sysex) => {
                    if sysex.chan == self.sysex_chan {
                        log::trace!("Received {:?}", sysex.proc);

                        return Ok(msg);
                    }

                    log::trace!("Ignoring sysex on {}: {:?}", sysex.chan, sysex.proc);
                }
            }
        }
    }

    /// Awaits for a device handshake response.
    ///
    /// Listens to all channels and returns the next `WhoAmIResp`.
    /// The channels returned by this message will be used in
    /// subsequent calls to [`Self::listen`].
    async fn await_handshake(&mut self) -> Result<Message, Error> {
        log::debug!("Awaiting WhoAmIResp");

        loop {
            let msg = self.handshake_receive().await?;

            if let Message::SysEx(sysex) = &msg {
                if let sysex::Message {
                    proc: Procedure::WhoAmIResp(resp),
                    ..
                } = sysex.as_ref()
                {
                    self.cc_chan = resp.transmit_chan;
                    self.sysex_chan = resp.sysex_chan;
                    self.state = ListenerState::FoundDevice;

                    log::info!(
                        "Found device. Got cc rx {} tx {} & sysex {}",
                        resp.receive_chan,
                        resp.transmit_chan,
                        resp.sysex_chan,
                    );

                    return Ok(msg);
                }
            }

            log::debug!("Ignoring {msg:?}");
        }
    }

    async fn receive(&mut self) -> Result<Message, Error> {
        let midi_msg = loop {
            futures::select_biased! {
                chan_res = self.chan_rx.recv_async() => {
                    let chan = chan_res.expect("Broken chan channel");
                    log::info!("Changing chan to {chan}");
                    self.cc_chan = chan;
                    self.sysex_chan = chan;
                }
                msg_res = self.msg_rx.recv_async() => break msg_res.expect("Broken message channel"),
            }
        };

        // Set to true to dump buffers
        if false {
            println!(
                "In Buffer {:?}\n",
                midi_msg
                    .iter()
                    .map(|byte| format!("x{byte:02x}"))
                    .collect::<Vec<String>>(),
            );
        }

        let (_, proc) = parse_raw_midi_msg(&midi_msg).map_err(|err| {
            log::error!("{}", err.to_string());

            Error::Parse
        })?;

        Ok(proc)
    }

    async fn handshake_receive(&mut self) -> Result<Message, Error> {
        use futures::FutureExt;

        let mut timeout = smol::Timer::after(HANDSHAKE_TIMEOUT).fuse();

        let midi_msg = futures::select_biased! {
            res = self.msg_rx.recv_async() => res.expect("Broken message channel"),
            _ = timeout => {
                log::debug!("timeout");
                return Err(Error::HandshakeTimeout);
            }
        };

        let (_, proc) = parse_raw_midi_msg(&midi_msg).map_err(|err| {
            log::error!("{}", err.to_string());

            Error::Parse
        })?;

        Ok(proc)
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(midi_in) = self.midi_in.take() {
            midi_in.close();
        }
    }
}
