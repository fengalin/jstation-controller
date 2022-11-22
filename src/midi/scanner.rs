use std::sync::Arc;

use crate::midi;

/// Trait for devices which can be found on input and output Midi ports.
pub trait Scannable {
    /// The type resulting from a sucessful connection to an input port.
    type In;
    /// The type resulting from a sucessful connection to an output port.
    type Out;

    /// The error type.
    type Error: std::error::Error;

    /// Get the input ports.
    fn ins(&self) -> &midi::PortsIn;

    /// Get the output ports.
    fn outs(&self) -> &midi::PortsOut;

    /// Attempts to connect both in and out ports at once.
    fn connect(
        &mut self,
        port_in: Arc<str>,
        port_out: Arc<str>,
    ) -> Result<(Self::In, Self::Out), Self::Error>;

    /// Attempts to connect in port.
    fn connect_in(&mut self, port_name: Arc<str>) -> Result<Self::In, Self::Error>;

    /// Attempts to connect out port.
    fn connect_out(&mut self, port_name: Arc<str>) -> Result<Self::Out, Self::Error>;
}

enum State {
    /// Scan using the same name for in and out ports.
    ///
    /// This should be sufficient in most cases and faster,
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

pub struct Context {
    state: State,
}

impl Context {
    pub fn new(scannable: &impl Scannable) -> Self {
        // We NEED to own the iterator here because
        // `ScannerContext` will be moved around and we don't want to
        // track the `scannable.outs` lifetime for this not efficient
        // sensitive feature.
        #[allow(clippy::needless_collect)]
        let out_ports: Vec<Arc<str>> = scannable.outs().list().collect();

        Context {
            state: State::SamePortNames {
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
    pub fn connect_next(mut self, scannable: &mut impl Scannable) -> Option<Self> {
        loop {
            use State::*;
            match self.state {
                SamePortNames {
                    ref mut port_name_iter,
                } => {
                    for port_name in port_name_iter.by_ref() {
                        if let Err(err) = scannable.connect(port_name.clone(), port_name.clone()) {
                            log::trace!("Skipping in/out port {port_name}: {err}");
                            continue;
                        }

                        return Some(self);
                    }

                    // Not found using same port name mode, switch to Cominations mode

                    // We NEED to own the iterator here because
                    // `ScannerContext` will be moved around and we don't want to
                    // track the `scannable.outs` lifetime for this not efficient
                    // sensitive feature.
                    #[allow(clippy::needless_collect)]
                    let port_out_names: Vec<Arc<str>> = scannable.outs().list().collect();
                    let port_in_names: Vec<Arc<str>> = scannable.ins().list().collect();
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
                            if let Err(err) = scannable.connect_in(port_in_name.clone()) {
                                log::trace!("Skipping in port {port_in_name}: {err}");
                                continue;
                            }

                            // Test this combination
                            return Some(self);
                        }

                        // Exhausted the port ins, try next port out, if any
                    }

                    loop {
                        if let Some(port_out_name) = port_out_name_iter.next() {
                            if let Err(err) = scannable.connect_out(port_out_name.clone()) {
                                log::trace!("Skipping out port {port_out_name}: {err}");
                                continue;
                            }

                            *cur_port_out_name = Some(port_out_name);

                            break;
                        } else {
                            // No more ports nor modes to test
                            log::warn!("Device not found on any ports combinations");

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
