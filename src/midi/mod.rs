pub mod channel_voice;
pub use channel_voice::{ChannelVoice, CC};

mod error;
pub use error::Error;

pub mod port;
pub use port::{DirectionalPorts, PortsIn, PortsOut};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Tag(u8);

impl Tag {
    pub const fn into_inner(self) -> u8 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Channel(u8);

impl Channel {
    pub const ALL: Self = Channel(0x7e);

    pub const fn into_inner(self) -> u8 {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct TagChannel {
    pub tag: Tag,
    pub chan: Channel,
}

impl From<u8> for TagChannel {
    fn from(tag_chan: u8) -> TagChannel {
        TagChannel {
            tag: Tag(tag_chan & 0xf0),
            chan: Channel(tag_chan & 0x0f),
        }
    }
}

pub mod sysex {
    pub const TAG: u8 = 0xf0;
    pub const END_TAG: u8 = 0xf7;
}
