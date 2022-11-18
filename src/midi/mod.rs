mod error;
pub use error::Error;

pub mod port;
pub use port::{DirectionalPorts, PortsIn, PortsOut};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Channel(u8);

impl Channel {
    pub const ALL: Self = Channel(0x7e);

    pub fn into_inner(self) -> u8 {
        self.0
    }
}

impl From<u8> for Channel {
    fn from(chan: u8) -> Self {
        Channel(chan)
    }
}

pub mod sysex {
    pub const TAG: u8 = 0xf0;
    pub const END_TAG: u8 = 0xf7;
}
