use crate::jstation::data::{ParameterSetter, RawValue};

pub trait BoolParameter:
    From<bool>
    + Into<bool>
    + ParameterSetter<Parameter = Self>
    + Default
    + Clone
    + Copy
    + Eq
    + PartialEq
{
    const TRUE: Self;
    const FALSE: Self;

    fn param_name(self) -> &'static str;

    fn from_raw(raw: RawValue) -> Self {
        (raw.as_u8() == 0).into()
    }

    fn raw_value(self) -> RawValue {
        RawValue::new(if self.into() { 0 } else { u8::MAX })
    }

    fn is_true(&self) -> bool {
        (*self).into()
    }

    /// Resets the parameter to its default value.
    fn reset(&mut self) -> Option<Self> {
        self.set(Self::default())
    }
}
