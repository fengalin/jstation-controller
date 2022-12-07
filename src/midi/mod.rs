pub mod channel_voice;
pub use channel_voice::{CCNumber, CCValue, ChannelVoice, CC};

mod error;
pub use error::Error;

pub mod port;
pub use port::{DirectionalPorts, PortsIn, PortsOut};

pub mod scanner;
pub use scanner::Scannable;

use std::fmt;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Tag(u8);

impl Tag {
    pub fn as_u8(self) -> u8 {
        self.0
    }
}

impl From<Tag> for u8 {
    fn from(tag: Tag) -> Self {
        tag.0
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&(self.0 + 1), f)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Channel(u8);

impl Channel {
    pub const ALL: Self = Channel(0x7e);

    pub fn as_u8(self) -> u8 {
        self.0
    }
}

impl From<u8> for Channel {
    fn from(chan: u8) -> Channel {
        Channel(chan)
    }
}

impl From<Channel> for u8 {
    fn from(chan: Channel) -> Self {
        chan.0
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&(self.0 + 1), f)
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
