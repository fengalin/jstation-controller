use std::{cell::Cell, sync::Arc};

use crate::{
    jstation::{self, Error, Listener, Message},
    midi,
};

/// An UI oriented decorator for [`crate::jstation::JStation`].
///
/// It mostly adds `iced` subscriptions handling.
pub struct JStation {
    inner: jstation::JStation,
    next_subscription_id: usize,
    subscription: Option<Subscription>,
}

impl JStation {
    pub fn new() -> Self {
        JStation {
            inner: jstation::JStation::new(crate::APP_NAME.clone()),
            next_subscription_id: 0,
            subscription: None,
        }
    }

    pub fn iface(&self) -> &jstation::Interface {
        self.inner.iface()
    }

    pub fn dsp(&self) -> &jstation::dsp::Dsp {
        self.inner.dsp()
    }

    pub fn cur_prog_id(&self) -> Option<jstation::ProgramId> {
        self.inner.cur_prog_id()
    }

    pub fn programs_bank(&self) -> jstation::ProgramsBank {
        self.inner.programs_bank()
    }

    pub fn get_program(&self, prog_id: jstation::ProgramId) -> Option<&jstation::Program> {
        self.inner.get_program(prog_id)
    }

    pub fn has_changed(&self) -> bool {
        self.inner.has_changed()
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        self.inner.iface_mut().refresh()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.subscription = None;
    }

    pub fn handle_device(&mut self, msg: Message) -> Result<(), Error> {
        self.inner.handle_device(msg)
    }

    pub fn change_program(&mut self, id: jstation::ProgramId) -> Result<(), Error> {
        self.inner.change_program(id)
    }

    pub fn store_to(&mut self, nb: jstation::ProgramNb) -> Result<(), Error> {
        self.inner.store_to(nb)
    }

    pub fn undo(&mut self) -> Result<(), Error> {
        self.inner.undo()
    }

    pub fn rename(&mut self, name: impl ToString) {
        self.inner.rename(name);
    }

    pub fn select_bank(&mut self, bank: jstation::ProgramsBank) {
        self.inner.select_bank(bank);
    }

    pub fn update_param(&mut self, param: jstation::dsp::Parameter) {
        self.inner.update_param(param);
    }

    pub fn update_utility_settings(&mut self, settings: jstation::dsp::UtilitySettings) {
        self.inner.update_utility_settings(settings);
    }

    fn set_listener(&mut self, listener: Listener) {
        self.subscription = Some(Subscription {
            id: self.next_subscription_id,
            listener: Cell::new(Some(listener)),
        });

        self.next_subscription_id += 1;
    }
}

impl midi::Scannable for JStation {
    type In = ();
    type Out = ();
    type Error = Error;

    fn ins(&self) -> &midi::PortsIn {
        &self.inner.iface().ins
    }

    fn outs(&self) -> &midi::PortsOut {
        &self.inner.iface().outs
    }

    fn connect(&mut self, port_in: Arc<str>, port_out: Arc<str>) -> Result<((), ()), Error> {
        let (listener, ()) = self.inner.iface_mut().connect(port_in, port_out)?;
        self.set_listener(listener);

        Ok(((), ()))
    }

    fn connect_in(&mut self, port_name: Arc<str>) -> Result<(), Error> {
        let listener = self.inner.iface_mut().connect_in(port_name)?;
        self.set_listener(listener);

        Ok(())
    }

    fn connect_out(&mut self, port_name: Arc<str>) -> Result<(), Error> {
        self.inner.iface_mut().connect_out(port_name)
    }
}

/// Scanner helpers.
impl JStation {
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
impl JStation {
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

        if self.inner.iface().is_connected() {
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
