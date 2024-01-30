use iced::futures::{self, channel::mpsc, StreamExt};
use std::{cell::Cell, sync::Arc};

use crate::{
    jstation::{self, Error, JStationImpl, Listener, Message},
    midi,
};

/// An UI oriented decorator for [`crate::jstation::JStation`].
///
/// It mostly adds `iced` subscriptions handling.
pub struct JStation {
    inner: jstation::JStation,
    listener_tx: mpsc::Sender<Listener>,
    // Needs interior mutability because of subscription(&self)
    listener_rx: Cell<Option<mpsc::Receiver<Listener>>>,
    /// The listener to use once both MIDI channels are connected
    pending_listener: Option<Listener>,
}

impl JStation {
    pub fn new() -> Self {
        let (listener_tx, listener_rx) = mpsc::channel(1);

        JStation {
            inner: jstation::JStation::new(crate::APP_NAME.clone()),
            listener_tx,
            listener_rx: Cell::new(Some(listener_rx)),
            pending_listener: None,
        }
    }

    fn set_listener(&mut self, listener: Listener) {
        self.pending_listener = Some(listener);
        self.maybe_listen();
    }

    fn maybe_listen(&mut self) {
        if self.inner.iface().is_connected() {
            // Only start listening if midi_out is connected
            // otherwise handshake would timeout for nothing.
            if let Some(listener) = self.pending_listener.take() {
                log::debug!("sending listener to subscription");

                self.listener_tx
                    .try_send(listener)
                    .expect("failed to send listener");
            }
        }
    }
}

impl JStationImpl for JStation {
    type Inner = jstation::JStation;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut Self::Inner {
        &mut self.inner
    }

    fn clear(&mut self) {
        self.inner.clear();
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
        self.inner.iface_mut().connect_out(port_name)?;
        self.maybe_listen();

        Ok(())
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

/// iced Subscription helper.
impl JStation {
    pub fn subscription(&self) -> iced::Subscription<Result<Message, Error>> {
        use futures::future::FutureExt;

        struct SubscriptionToken {
            listener: Option<Listener>,
            listener_rx: mpsc::Receiver<Listener>,
        }

        async fn iface_subscription(
            mut token: Option<SubscriptionToken>,
        ) -> (Result<Message, Error>, Option<SubscriptionToken>) {
            let SubscriptionToken {
                ref mut listener_rx,
                ref mut listener,
            } = token
                .as_mut()
                .expect("token available while subscription is unfolded");

            let msg_res = loop {
                let listener_rx_opt = {
                    let listener_fut = async {
                        if let Some(listener) = listener.as_mut() {
                            log::trace!("Awaiting device message");
                            listener.listen().await
                        } else {
                            futures::future::pending().await
                        }
                    }
                    .fuse();

                    smol::pin!(listener_fut);

                    futures::select_biased! {
                        listener_rx_opt = listener_rx.next() => listener_rx_opt,
                        msg_res = listener_fut => break msg_res,
                    }
                };

                if let Some(new_listener) = listener_rx_opt {
                    log::debug!("Got new listener");
                    *listener = Some(new_listener);
                } else {
                    log::info!("Subscription listener channel closed");
                    let () = futures::future::pending().await;
                }
            };

            if let Err(err) = msg_res.as_ref() {
                if err.is_handshake_timeout() {
                    // Device not found using this listener configuration,
                    log::trace!("Discarding listener due to handshake timeout");

                    *listener = None;
                }
            }

            (msg_res, token)
        }

        if let Some(listener_rx) = self.listener_rx.take() {
            log::debug!("Spawning device subscription");

            return iced::subscription::unfold(
                std::any::TypeId::of::<SubscriptionToken>(),
                Some(SubscriptionToken {
                    listener: None,
                    listener_rx,
                }),
                iface_subscription,
            );
        }

        // Keep subscription running
        iced::subscription::unfold(
            std::any::TypeId::of::<SubscriptionToken>(),
            None,
            iface_subscription,
        )
    }
}
