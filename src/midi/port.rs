use std::{collections::BTreeMap, fmt, sync::Arc};

use super::Error;

pub type PortsIn = DirectionalPorts<midir::MidiInput>;
pub type PortsOut = DirectionalPorts<midir::MidiOutput>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    In,
    Out,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Direction {
    pub fn idx(self) -> usize {
        match self {
            Direction::In => 0,
            Direction::Out => 1,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Direction::In => "In Port",
            Direction::Out => "Out Port",
        }
    }
}

pub struct DirectionalPorts<IO: midir::MidiIO> {
    map: BTreeMap<Arc<str>, IO::Port>,
    cur: Option<Arc<str>>,
    client_name: Arc<str>,
}

impl<IO: midir::MidiIO> DirectionalPorts<IO> {
    pub fn list(&self) -> impl Iterator<Item = Arc<str>> + '_ {
        self.map.keys().cloned()
    }

    pub fn cur(&self) -> Option<Arc<str>> {
        self.cur.as_ref().cloned()
    }

    pub fn disconnect(&mut self) {
        if let Some(cur) = self.cur.take() {
            log::debug!("Disconnected Input from {}", cur);
        }
    }

    fn refresh_from(&mut self, conn: IO) -> Result<(), Error> {
        self.map.clear();

        let mut prev = self.cur.take();
        for port in conn.ports().iter() {
            let name = conn.port_name(port)?;
            if !name.starts_with(self.client_name.as_ref()) {
                if let Some(ref prev_ref) = prev {
                    if prev_ref.as_ref() == name {
                        self.cur = prev.take();
                    }
                }

                self.map.insert(name.into(), port.clone());
            }
        }

        Ok(())
    }
}

impl PortsIn {
    pub fn new(client_name: Arc<str>) -> Self {
        Self {
            map: BTreeMap::new(),
            cur: None,
            client_name,
        }
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        let temp_conn = midir::MidiInput::new(&format!("{} referesh In ports", self.client_name))?;

        self.refresh_from(temp_conn)?;

        Ok(())
    }

    pub fn connect<D, C>(
        &mut self,
        port_name: Arc<str>,
        data: D,
        callback: C,
    ) -> Result<midir::MidiInputConnection<D>, Error>
    where
        D: Send,
        C: FnMut(u64, &[u8], &mut D) + Send + 'static,
    {
        let port = self
            .map
            .get(&port_name)
            .ok_or_else(|| Error::PortNotFound(port_name.clone()))?
            .clone();

        let midi_conn = midir::MidiInput::new(&self.client_name)?
            .connect(&port, &port_name, callback, data)
            .map_err(|_| {
                self.cur = None;
                Error::PortConnection
            })?;

        log::debug!("Connected for Input to {}", port_name);
        self.cur = Some(port_name);

        Ok(midi_conn)
    }
}

impl PortsOut {
    pub fn new(client_name: Arc<str>) -> Self {
        Self {
            map: BTreeMap::new(),
            cur: None,
            client_name,
        }
    }

    pub fn refresh(&mut self) -> Result<(), Error> {
        let temp_conn =
            midir::MidiOutput::new(&format!("{} referesh Out ports", self.client_name,))?;

        self.refresh_from(temp_conn)?;

        Ok(())
    }

    pub fn connect(&mut self, port_name: Arc<str>) -> Result<midir::MidiOutputConnection, Error> {
        let port = self
            .map
            .get(&port_name)
            .ok_or_else(|| Error::PortNotFound(port_name.clone()))?
            .clone();

        let midi_conn = midir::MidiOutput::new(&self.client_name)?
            .connect(&port, &port_name)
            .map_err(|_| {
                self.cur = None;
                Error::PortConnection
            })?;

        log::debug!("Connected for Output to {}", port_name);
        self.cur = Some(port_name);

        Ok(midi_conn)
    }
}
