use nom::IResult;

use std::fmt;

use crate::midi;

#[derive(Copy, Clone, Debug)]
pub struct CC {
    pub nb: CCNumber,
    pub value: CCValue,
}

impl CC {
    pub const TAG: midi::Tag = midi::Tag(0xb0);

    pub fn new(nb: CCNumber, value: CCValue) -> Self {
        CC { nb, value }
    }

    pub fn build_for(self, chan: midi::Channel) -> [u8; 3] {
        let tag_chan = midi::TagChannel {
            tag: Self::TAG,
            chan,
        };

        [tag_chan.into(), self.nb.into(), self.value.into()]
    }

    pub fn parse(i: &[u8]) -> IResult<&[u8], CC> {
        use nom::{
            bytes::complete::take,
            error::{self, Error},
        };

        let (i, nb) = take(1usize)(i)?;
        let nb = CCNumber::try_from(nb[0]).map_err(|err| {
            log::error!("CC: {err}");

            nom::Err::Failure(Error::new(i, error::ErrorKind::Verify))
        })?;

        let (i, value) = take(1usize)(i)?;
        let value = CCValue::try_from(value[0]).map_err(|err| {
            log::error!("CC: {err}");

            nom::Err::Failure(Error::new(i, error::ErrorKind::Verify))
        })?;

        Ok((i, CC { nb, value }))
    }
}

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum Error {
    #[error("CC number out of range 0x{:02x}", .0)]
    NumberOutOfRange(u8),
    #[error("CC value out of range 0x{:02x}", .0)]
    ValueOutOfRange(u8),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct CCNumber(u8);

impl CCNumber {
    pub const MAX: CCNumber = CCNumber(0x77);

    /// Builds a new CC `CCNumber`.
    ///
    /// # Panic
    ///
    /// Panics if the provided number is larger than `CCNumber::MAX`.
    pub const fn new(nb: u8) -> Self {
        assert!(nb <= Self::MAX.0);

        CCNumber(nb)
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for CCNumber {
    type Error = Error;

    fn try_from(nb: u8) -> Result<Self, Self::Error> {
        if nb > Self::MAX.0 {
            return Err(Error::NumberOutOfRange(nb));
        }

        Ok(Self(nb))
    }
}

impl From<CCNumber> for u8 {
    fn from(nb: CCNumber) -> Self {
        nb.0
    }
}

impl fmt::Display for CCNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct CCValue(u8);

impl CCValue {
    pub const ZERO: CCValue = CCValue(0);
    pub const MAX: CCValue = CCValue(0x7f);

    pub const fn new_clipped(value: u8) -> Self {
        CCValue(if value > Self::MAX.0 {
            Self::MAX.0
        } else {
            value
        })
    }

    pub const fn as_u8(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for CCValue {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::MAX.0 {
            return Err(Error::ValueOutOfRange(value));
        }

        Ok(Self(value))
    }
}

impl From<CCValue> for u8 {
    fn from(value: CCValue) -> Self {
        value.0
    }
}

impl From<&CCValue> for u8 {
    fn from(value: &CCValue) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::{CCNumber, CCValue, Error};

    #[test]
    fn parameter_number() {
        assert_eq!(
            CCNumber::try_from(CCNumber::MAX.0 - 1),
            Ok(CCNumber(CCNumber::MAX.0 - 1))
        );
        assert_eq!(CCNumber::try_from(CCNumber::MAX.0), Ok(CCNumber::MAX));

        assert_eq!(
            CCNumber::try_from(CCNumber::MAX.0 + 1),
            Err(Error::NumberOutOfRange(CCNumber::MAX.0 + 1))
        );
        assert_eq!(CCNumber::try_from(0xff), Err(Error::NumberOutOfRange(0xff)));
    }

    #[test]
    fn value() {
        assert_eq!(
            CCValue::try_from(CCValue::MAX.0 - 1),
            Ok(CCValue(CCValue::MAX.0 - 1))
        );
        assert_eq!(CCValue::try_from(CCValue::MAX.0), Ok(CCValue::MAX));

        assert_eq!(
            CCValue::try_from(CCValue::MAX.0 + 1),
            Err(Error::ValueOutOfRange(CCValue::MAX.0 + 1))
        );
        assert_eq!(CCValue::try_from(0xff), Err(Error::ValueOutOfRange(0xff)));
    }
}
