use nom::{
    bytes::complete::take,
    error::{self, Error},
    IResult,
};

use crate::midi;

#[derive(Copy, Clone, Debug)]
pub struct CC {
    pub param: Parameter,
    pub value: Value,
}

impl CC {
    pub const TAG: midi::Tag = midi::Tag(0xb0);
}

pub fn parse(i: &[u8]) -> IResult<&[u8], CC> {
    let (i, param) = take(1usize)(i)?;
    let param = Parameter::try_from(param[0]).map_err(|err| {
        log::error!("CC: {err}");

        nom::Err::Failure(Error::new(i, error::ErrorKind::Verify))
    })?;

    let (i, value) = take(1usize)(i)?;
    let value = Value::try_from(value[0]).map_err(|err| {
        log::error!("CC: {err}");

        nom::Err::Failure(Error::new(i, error::ErrorKind::Verify))
    })?;

    Ok((i, CC { param, value }))
}

#[derive(Debug, thiserror::Error)]
pub enum CCError {
    #[error("J-Station CC parameter out of range 0x{:02x}", .0)]
    ParameterOutOfRange(u8),
    #[error("J-Station CC value out of range 0x{:02x}", .0)]
    ValueOutOfRange(u8),
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Parameter(u8);

impl Parameter {
    pub const MAX: Parameter = Parameter(0x77);
}

impl TryFrom<u8> for Parameter {
    type Error = CCError;

    fn try_from(param: u8) -> Result<Self, Self::Error> {
        if param > Self::MAX.0 {
            return Err(CCError::ParameterOutOfRange(param));
        }

        Ok(Self(param))
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Value(u8);

impl Value {
    pub const MAX: Value = Value(0x7f);
}

impl TryFrom<u8> for Value {
    type Error = CCError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::MAX.0 {
            return Err(CCError::ValueOutOfRange(value));
        }

        Ok(Self(value))
    }
}
