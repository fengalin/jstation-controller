use std::fmt;

use crate::{jstation::data::ParameterNumber, midi};

#[derive(Copy, Clone, Debug)]
pub struct RawParameter {
    nb: ParameterNumber,
    value: RawValue,
}

impl RawParameter {
    pub fn new(nb: ParameterNumber, value: RawValue) -> Self {
        RawParameter { nb, value }
    }

    pub fn nb(self) -> ParameterNumber {
        self.nb
    }

    pub fn value(self) -> RawValue {
        self.value
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct RawValue(u8);

impl RawValue {
    pub const ZERO: Self = RawValue(0);
    pub const CENTER: Self = RawValue(0x7f);
    pub const MAX: Self = RawValue(0xff);

    pub const fn new(value: u8) -> Self {
        RawValue(value)
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl From<u8> for RawValue {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<midi::Channel> for RawValue {
    fn from(chan: midi::Channel) -> Self {
        Self(chan.as_u8())
    }
}

impl From<RawValue> for u8 {
    fn from(value: RawValue) -> Self {
        value.0
    }
}

impl From<&RawValue> for u8 {
    fn from(value: &RawValue) -> Self {
        value.0
    }
}

impl fmt::Display for RawValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
