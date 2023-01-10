use std::{cell::Cell, sync::Arc};

use crate::{
    jstation::{self, procedure, Error, Listener, Message, Program},
    midi,
};

/// An UI oriented decorator for [`crate::jstation::Interface`].
///
/// It mostly adds `iced` subscriptions handling.
pub struct Interface {
    iface: jstation::Interface,
    next_subscription_id: usize,
    subscription: Option<Subscription>,
}

impl Interface {
    pub fn new() -> Self {
        Interface {
            iface: jstation::Interface::new(crate::APP_NAME.clone()),
            next_subscription_id: 0,
            subscription: None,
        }
    }

    pub fn iface(&self) -> &jstation::Interface {
        &self.iface
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        self.iface.refresh()
    }

    pub fn clear(&mut self) {
        self.iface.clear();
        self.subscription = None;
    }

    pub fn have_who_am_i_resp(&mut self, resp: procedure::WhoAmIResp) -> Result<(), Error> {
        self.iface.have_who_am_i_resp(resp)
    }

    pub fn request_utility_settings(&mut self) -> Result<(), Error> {
        self.iface.request_utility_settings()
    }

    pub fn bank_dump(&mut self) -> Result<(), Error> {
        self.iface.bank_dump()
    }

    pub fn program_update_req(&mut self) -> Result<(), Error> {
        self.iface.program_update_req()
    }

    pub fn change_program(&mut self, id: impl Into<midi::ProgramNumber>) -> Result<(), Error> {
        self.iface.change_program(id)
    }

    pub fn request_program(&mut self, id: jstation::ProgramId) -> Result<(), Error> {
        self.iface.request_program(id)
    }

    pub fn reload_program(&mut self) -> Result<(), Error> {
        self.iface.reload_program()
    }

    pub fn store_program(&mut self, prog: &Program) -> Result<(), Error> {
        self.iface.store_program(prog)
    }

    pub fn send_cc(&mut self, cc: midi::CC) -> Result<(), Error> {
        self.iface.send_cc(cc)
    }

    fn set_listener(&mut self, listener: Listener) {
        self.subscription = Some(Subscription {
            id: self.next_subscription_id,
            listener: Cell::new(Some(listener)),
        });

        self.next_subscription_id += 1;
    }

    pub fn connected_ports(&self) -> Option<(Arc<str>, Arc<str>)> {
        self.iface.connected_ports()
    }
}

impl midi::Scannable for Interface {
    type In = ();
    type Out = ();
    type Error = Error;

    fn ins(&self) -> &midi::PortsIn {
        &self.iface.ins
    }

    fn outs(&self) -> &midi::PortsOut {
        &self.iface.outs
    }

    fn connect(&mut self, port_in: Arc<str>, port_out: Arc<str>) -> Result<((), ()), Error> {
        let (listener, ()) = self.iface.connect(port_in, port_out)?;
        self.set_listener(listener);

        Ok(((), ()))
    }

    fn connect_in(&mut self, port_name: Arc<str>) -> Result<(), Error> {
        let listener = self.iface.connect_in(port_name)?;
        self.set_listener(listener);

        Ok(())
    }

    fn connect_out(&mut self, port_name: Arc<str>) -> Result<(), Error> {
        self.iface.connect_out(port_name)
    }
}

/// Scanner helpers.
impl Interface {
    pub fn start_scan(&mut self) -> Option<midi::scanner::Context> {
        midi::scanner::Context::new(self).connect_next(self)
    }

    pub fn scan_next(&mut self, ctx: midi::scanner::Context) -> Option<midi::scanner::Context> {
        ctx.connect_next(self)
    }
}

struct Subscription {
    id: usize,
    // Need iterior mutability because of `subscription(&self)`
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
                        // Subscription stream will return `None` at next iteration.
                        log::trace!("Cancelling listener subscription due to handshake timeout");

                        return Some((res, None));
                    }
                }

                Some((res, listener))
            } else {
                None
            }
        }

        if self.iface.is_connected() {
            // Only listen if midi_out is connected
            // otherwise handshake would timeout for nothing.
            if let Some(subscription) = self.subscription.as_ref() {
                let listener = subscription.listener.take();
                if listener.is_some() {
                    log::trace!("Spawning new listener with id {}", subscription.id);
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
