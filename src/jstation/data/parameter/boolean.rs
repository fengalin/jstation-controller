use crate::jstation::data::{ParameterNumber, RawParameter, RawValue};

pub trait BoolParameter:
    From<bool> + Into<bool> + PartialEq + Copy + Clone + std::fmt::Debug + Sized
{
    const NAME: &'static str;
    const DEFAULT: bool;

    fn from_raw(raw: RawValue) -> Self {
        (raw.as_u8() == 0).into()
    }

    fn to_raw_value(self) -> RawValue {
        RawValue::new(if self.into() { 0 } else { u8::MAX })
    }

    fn is_active(self) -> bool {
        self.into()
    }

    /// Resets the parameter to its default value.
    fn reset(&mut self) -> Option<Self> {
        self.set(Self::DEFAULT.into())
    }

    /// Sets the value if it is different than current.
    fn set(&mut self, new: Self) -> Option<Self> {
        if new == *self {
            return None;
        }

        *self = new;

        Some(new)
    }
}

/// A `BoolParameter`, view as a raw `Parameter`, e.g. part of a `Program` `data`.
pub trait BoolRawParameter: BoolParameter {
    const PARAMETER_NB: ParameterNumber;

    fn to_raw_parameter(self) -> RawParameter {
        RawParameter::new(Self::PARAMETER_NB, self.to_raw_value())
    }
}
