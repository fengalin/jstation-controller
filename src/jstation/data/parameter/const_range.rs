use crate::jstation::{
    data::{Normal, ParameterSetter, RawValue},
    Error,
};

pub trait ConstRangeParameter: ParameterSetter<Parameter = Self> + Clone + Copy {
    const NAME: &'static str;
    const DEFAULT: Normal;
    const MIN_RAW: RawValue;
    const MAX_RAW: RawValue;
    const RANGE: crate::jstation::data::DiscreteRange;

    fn from_snapped(normal: Normal) -> Self;
    fn try_from_raw(raw: RawValue) -> Result<Self, Error>;
}
