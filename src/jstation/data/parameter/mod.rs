#[macro_use]
mod boolean;
pub use boolean::{BoolCCParameter, BoolParameter, BoolRawParameter};

#[macro_use]
mod discrete;
pub use discrete::{
    DiscreteCCParameter, DiscreteParameter, DiscreteRange, DiscreteRawParameter, DiscreteValue,
};

mod normal;
pub use normal::Normal;

mod raw;
pub use raw::{RawParameter, RawValue};

use std::fmt;

use crate::jstation::Error;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct ParameterNumber(u8);

impl ParameterNumber {
    pub const MAX: ParameterNumber = ParameterNumber(43);

    pub const fn new(nb: u8) -> Self {
        ParameterNumber(nb)
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for ParameterNumber {
    type Error = Error;

    fn try_from(nb: u8) -> Result<Self, Error> {
        if nb > Self::MAX.0 {
            return Err(Error::ParameterNumberOutOfRange(nb));
        }

        Ok(ParameterNumber(nb))
    }
}

// FIXME should probably be faillible
impl From<ParameterNumber> for u8 {
    fn from(nb: ParameterNumber) -> Self {
        nb.0
    }
}

impl fmt::Display for ParameterNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&(self.0 + 1), f)
    }
}

pub enum ValueStatus {
    Changed,
    Unchanged,
}

impl ValueStatus {
    pub fn has_changed(self) -> bool {
        matches!(self, ValueStatus::Changed)
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, ParameterNumber};

    #[test]
    fn parameter_number() {
        assert_eq!(
            ParameterNumber::try_from(ParameterNumber::MAX.0 - 1).unwrap(),
            ParameterNumber(ParameterNumber::MAX.0 - 1),
        );

        assert_eq!(
            ParameterNumber::try_from(ParameterNumber::MAX.0).unwrap(),
            ParameterNumber(ParameterNumber::MAX.0),
        );

        if let Error::ParameterNumberOutOfRange(nb) =
            ParameterNumber::try_from(ParameterNumber::MAX.0 + 1).unwrap_err()
        {
            assert_eq!(nb, ParameterNumber::MAX.0 + 1)
        }

        let res = ParameterNumber::try_from(0xff);
        if let Error::ParameterNumberOutOfRange(nb) = res.unwrap_err() {
            assert_eq!(nb, 0xff)
        }
    }
}
