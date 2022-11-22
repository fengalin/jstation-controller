use std::{cell::Cell, sync::Arc};

use crate::{
    jstation::{parse_raw_midi_msg, procedure, sysex, Message, Procedure, ProcedureBuilder},
    midi,
};

use std::time::Duration;

const HANDSHAKE_TIMEOUT: Duration = Duration::from_millis(200);

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Error Parsing MIDI message")]
    Parse,
    #[error("{}", .0)]
    Midi(#[from] midi::Error),
    #[error("No MIDI connection")]
    MidiNotConnected,
    #[error("An error occured sending a MIDI message")]
    MidiSend,
    #[error("Device handshake timed out")]
    HandshakeTimeout,
}

impl Error {
    pub fn is_handshake_timeout(&self) -> bool {
        matches!(self, Error::HandshakeTimeout)
    }
}

pub struct Interface {
    pub ins: midi::PortsIn,
    pub outs: midi::PortsOut,
    midi_out: Option<midir::MidiOutputConnection>,
    cc_chan: midi::Channel,
    sysex_chan: midi::Channel,
    // FIXME the following fields should be part of a wrapper under the ui module
    next_subscription_id: usize,
    subscription: Option<Subscription>,
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
            next_subscription_id: 0,
            subscription: None,
        }
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        self.ins.refresh()?;
        self.outs.refresh()?;

        Ok(())
    }

    pub fn clear(&mut self) {
        self.ins.disconnect();
        self.subscription = None;

        self.outs.disconnect();
        if let Some(midi_out) = self.midi_out.take() {
            midi_out.close();
        }
    }

    pub fn have_who_am_i_resp(&mut self, resp: &procedure::WhoAmIResp) -> Result<(), Error> {
        // FIXME check that this is the right channel to send cc
        self.cc_chan = resp.receive_chan;
        self.sysex_chan = resp.sysex_chan;

        log::debug!("Sending UtilitySettingsReq");
        let res = self.send_sysex(procedure::UtilitySettingsReq);
        if res.is_err() {
            // FIXME should probably assume connection is broken
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

    fn start_handshake(&mut self, midi_out: &mut midir::MidiOutputConnection) -> Result<(), Error> {
        self.cc_chan = midi::Channel::ALL;
        self.sysex_chan = midi::Channel::ALL;

        log::debug!("Sending WhoAmIReq");
        midi_out
            .send(&procedure::WhoAmIReq::default().build_for(midi::Channel::ALL))
            .map_err(|_| Error::MidiSend)
    }

    fn set_listener(&mut self, listener: Listener) {
        self.subscription = Some(Subscription {
            id: self.next_subscription_id,
            listener: Cell::new(Some(listener)),
        });

        self.next_subscription_id += 1;
    }

    pub fn connect(&mut self, port_in: Arc<str>, port_out: Arc<str>) -> Result<(), Error> {
        let mut midi_out = self.outs.connect(port_out)?;
        let listener = Listener::try_new(self, port_in)?;

        self.start_handshake(&mut midi_out)?;

        self.midi_out = Some(midi_out);
        self.set_listener(listener);

        Ok(())
    }

    pub fn connect_out(&mut self, port_name: Arc<str>) -> Result<(), Error> {
        let mut midi_out = self.outs.connect(port_name)?;

        if let Some(prev_midi_out) = self.midi_out.take() {
            prev_midi_out.close();
        }

        self.start_handshake(&mut midi_out)?;

        self.midi_out = Some(midi_out);

        Ok(())
    }

    pub fn connect_in(&mut self, port_name: Arc<str>) -> Result<(), Error> {
        let listener = Listener::try_new(self, port_name)?;

        self.cc_chan = midi::Channel::ALL;
        self.sysex_chan = midi::Channel::ALL;

        self.set_listener(listener);

        Ok(())
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

#[derive(Debug)]
enum ListenerState {
    AwaitingHandshake,
    FoundDevice,
}

struct Listener {
    state: ListenerState,
    cc_chan: midi::Channel,
    sysex_chan: midi::Channel,
    msg_rx: flume::Receiver<Vec<u8>>,
    midi_in: Option<midir::MidiInputConnection<flume::Sender<Vec<u8>>>>,
}

impl Listener {
    fn try_new(iface: &mut Interface, port_in: Arc<str>) -> Result<Self, Error> {
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
        })
    }

    async fn listen(&mut self) -> Result<Message, Error> {
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
                        log::debug!("Received {:?}", cv.msg);

                        return Ok(msg);
                    }

                    log::debug!("Ignoring channel voice on {:?}: {:?}", cv.chan, cv.msg);
                }
                SysEx(sysex) => {
                    // FIXME check whether this is emited when the chans change
                    // e.g. when user changes Chan within Utility Settings
                    // Hint: it seems that with produces Proc 0x23
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
                    // FIXME is this the one to listen to?
                    self.cc_chan = resp.transmit_chan;
                    self.sysex_chan = resp.sysex_chan;
                    self.state = ListenerState::FoundDevice;

                    log::debug!(
                        "Found device. Using cc rx {:?} tx {:?} & sysex {:?}",
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
        let midi_msg = self
            .msg_rx
            .recv_async()
            .await
            .expect("Broken internal channel");

        parse_raw_midi_msg(&midi_msg)
            .map(|(_, proc)| proc)
            .map_err(|err| {
                log::error!("{}", err.to_string());

                Error::Parse
            })
    }

    async fn handshake_receive(&mut self) -> Result<Message, Error> {
        use futures::FutureExt;

        let mut timeout = smol::Timer::after(HANDSHAKE_TIMEOUT).fuse();

        let midi_msg = futures::select_biased! {
            res = self.msg_rx.recv_async() => res.expect("Broken internal channel"),
            _ = timeout => {
                log::debug!("timeout");
                return Err(Error::HandshakeTimeout);
            }
        };

        parse_raw_midi_msg(&midi_msg)
            .map(|(_, proc)| proc)
            .map_err(|err| {
                log::error!("{}", err.to_string());

                Error::Parse
            })
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(midi_in) = self.midi_in.take() {
            midi_in.close();
        }
    }
}

struct Subscription {
    id: usize,
    // since they are solely ui related
    // Need iterior mutability because of subscription(&self)
    listener: Cell<Option<Listener>>,
}

/// iced Subscription helper.
impl Interface {
    pub fn subscription(&self) -> iced::Subscription<Result<Message, Error>> {
        async fn iface_subscription(
            mut listener: Option<Listener>,
        ) -> Option<(Result<Message, Error>, Option<Listener>)> {
            if let Some(listener_mut) = listener.as_mut() {
                let res = listener_mut.listen().await;
                if let Err(err) = res.as_ref() {
                    if err.is_handshake_timeout() {
                        // Device not found using this listener configuration,
                        // Subscription stream will return None at next iteration.
                        log::debug!("Cancelling listener subscription due to handshake timeout");

                        return Some((res, None));
                    }
                }

                Some((res, listener))
            } else {
                None
            }
        }

        if self.midi_out.is_some() {
            // Only listen if midi_out is connected
            // otherwise handshake would timeout for nothing.
            if let Some(subscription) = self.subscription.as_ref() {
                let listener = subscription.listener.take();
                if listener.is_some() {
                    log::debug!("Spawning new listener with id {}", subscription.id);
                }

                return iced::subscription::unfold(
                    crate::app::Subscription::JStation(subscription.id),
                    listener,
                    iface_subscription,
                );
            }
        }

        iced::Subscription::none()
    }
}

enum ScannerState {
    /// Scan using the same name for in and out ports.
    ///
    /// This should be sufficiant in most cases and faster,
    /// than trying all combinations.
    SamePortNames {
        port_name_iter: std::vec::IntoIter<Arc<str>>,
    },
    /// Scan using different names for in and out ports.
    ///
    /// This is used when scanning with the same name for
    /// in and out ports failed.
    Combinations {
        port_out_name_iter: std::vec::IntoIter<Arc<str>>,
        cur_port_out_name: Option<Arc<str>>,
        port_in_names: Vec<Arc<str>>,
        port_in_name_iter: std::vec::IntoIter<Arc<str>>,
    },
}

pub struct ScannerContext {
    state: ScannerState,
}

impl ScannerContext {
    fn new(iface: &Interface) -> Self {
        // We NEED to own the iterator here because
        // `ScannerContext` will be moved around and we don't want to
        // track the `iface.outs` lifetime for this not efficient
        // sensitive feature.
        #[allow(clippy::needless_collect)]
        let out_ports: Vec<Arc<str>> = iface.outs.list().collect();

        ScannerContext {
            state: ScannerState::SamePortNames {
                port_name_iter: out_ports.into_iter(),
            },
        }
    }

    /// Attempt to connect to next ports.
    ///
    /// Attempt to connect to next ports by iterating on all ports using
    /// the same port name mode, then the port name combination mode.
    ///
    /// Returns `None`, if no more ports can be tested.
    fn connect_next(mut self, iface: &mut Interface) -> Option<Self> {
        loop {
            use ScannerState::*;
            match self.state {
                SamePortNames {
                    ref mut port_name_iter,
                } => {
                    for port_name in port_name_iter.by_ref() {
                        if let Err(err) = iface.connect(port_name.clone(), port_name.clone()) {
                            log::debug!("Skipping in/out port {port_name}: {err}");
                            continue;
                        }

                        return Some(self);
                    }

                    // Not found using same port name mode, switch to Cominations mode

                    // We NEED to own the iterator here because
                    // `ScannerContext` will be moved around and we don't want to
                    // track the `iface.outs` lifetime for this not efficient
                    // sensitive feature.
                    #[allow(clippy::needless_collect)]
                    let port_out_names: Vec<Arc<str>> = iface.outs.list().collect();
                    let port_in_names: Vec<Arc<str>> = iface.ins.list().collect();
                    let port_in_name_iter = port_in_names.clone().into_iter();

                    self.state = Combinations {
                        port_out_name_iter: port_out_names.into_iter(),
                        cur_port_out_name: None,
                        port_in_names,
                        port_in_name_iter,
                    };
                }
                Combinations {
                    ref mut port_out_name_iter,
                    ref mut cur_port_out_name,
                    ref port_in_names,
                    ref mut port_in_name_iter,
                } => {
                    if let Some(port_out_name) = cur_port_out_name.as_ref() {
                        for port_in_name in port_in_name_iter
                            .filter(|port_in_name| port_out_name != port_in_name)
                            .by_ref()
                        {
                            if let Err(err) = iface.connect_in(port_in_name.clone()) {
                                log::debug!("Skipping in port {port_in_name}: {err}");
                                continue;
                            }

                            // Test this combination
                            return Some(self);
                        }

                        // Exhausted the port ins, try next port out, if any
                    }

                    loop {
                        if let Some(port_out_name) = port_out_name_iter.next() {
                            if let Err(err) = iface.connect_out(port_out_name.clone()) {
                                log::debug!("Skipping out port {port_out_name}: {err}");
                                continue;
                            }

                            *cur_port_out_name = Some(port_out_name);

                            break;
                        } else {
                            // No more ports nor modes to test
                            log::debug!("Device not found on any ports combinations");

                            return None;
                        }
                    }

                    *port_in_name_iter = port_in_names.clone().into_iter();
                    // Will try to connect to first port_in in next iteration
                }
            }
        }
    }
}

/// Scanner helpers.
impl Interface {
    pub fn start_scan(&mut self) -> Option<ScannerContext> {
        ScannerContext::new(self).connect_next(self)
    }

    pub fn scan_next(&mut self, ctx: ScannerContext) -> Option<ScannerContext> {
        ctx.connect_next(self)
    }
}
