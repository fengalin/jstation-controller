use crate::jstation::{
    data::{Normal, ParameterNumber, ParameterSetter, RawParameter, RawValue},
    Error,
};

pub trait DiscreteParameter: ParameterSetter<Parameter = Self> + Clone + Copy {
    fn name(self) -> &'static str;

    fn normal_default(self) -> Option<Normal>;

    fn normal(self) -> Option<Normal>;

    fn to_raw_value(self) -> Option<RawValue>;

    /// Resets the parameter to its default value.
    fn reset(&mut self) -> Option<Self>;

    fn is_active(self) -> bool {
        true
    }
}

// FIXME might want to define a generic RawParameter similar to CCParameter
/// A `DiscreteParameter`, view as a raw `Parameter`, e.g. part of a `Program` `data`.
pub trait DiscreteRawParameter: DiscreteParameter {
    const PARAMETER_NB: ParameterNumber;

    fn to_raw_parameter(self) -> Option<RawParameter> {
        self.to_raw_value()
            .map(|value| RawParameter::new(Self::PARAMETER_NB, value))
    }
}

// A discrete value which is guaranteed to be snapped to the provided [`DiscreteRange`].
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct DiscreteValue {
    normal: Normal,
}

impl DiscreteValue {
    pub fn new(normal: Normal, range: DiscreteRange) -> Self {
        DiscreteValue {
            normal: range.snap(normal),
        }
    }

    pub fn try_from_raw(raw: RawValue, range: DiscreteRange) -> Result<Self, Error> {
        let normal = range.try_normalize(raw)?;

        Ok(DiscreteValue { normal })
    }

    pub fn normal(self) -> Normal {
        self.normal
    }

    pub fn to_raw(self, range: DiscreteRange) -> RawValue {
        range.to_raw(self.normal)
    }
}

impl From<DiscreteValue> for Normal {
    fn from(val: DiscreteValue) -> Self {
        val.normal
    }
}

/// An inclusive `DiscreteRange` of discrete [`Value`]s between 0 and `max`.
///
/// [`Value`]: ../raw/struct.Value.html
#[derive(Clone, Copy, Debug)]
pub struct DiscreteRange {
    min: u8,
    delta: u8,
}

impl DiscreteRange {
    /// Builds a new `DiscreteRange` from the provided value.
    pub const fn new(min: RawValue, max: RawValue) -> Self {
        let min = min.as_u8();
        let max = max.as_u8();

        if min >= max {
            panic!("min >= max");
        }

        DiscreteRange {
            min,
            delta: max - min,
        }
    }

    fn out_of_range_error(self, value: impl Into<u8>) -> Error {
        Error::ValueOutOfRange {
            value: value.into(),
            min: self.min,
            max: self.delta + self.min,
        }
    }

    /// Tries to build a `Normal` from the provided `value`.
    ///
    /// The value must fit in this `DiscreteRange`.
    fn try_normalize(self, value: RawValue) -> Result<Normal, Error> {
        let zero_based_value = value
            .as_u8()
            .checked_sub(self.min)
            .ok_or_else(|| self.out_of_range_error(value))?;

        Normal::try_normalize(zero_based_value, self.delta)
            .map_err(|_| self.out_of_range_error(value))
    }

    pub fn to_raw(self, normal: Normal) -> RawValue {
        let min_based_value = normal.map_to(self.delta) + self.min;

        RawValue::new(min_based_value)
    }

    /// Returns a `Normal` from the provided value after snapping it to this `DiscreteRange`.
    pub fn snap(self, normal: Normal) -> Normal {
        normal.snap_to(self.delta)
    }
}

#[cfg(test)]
mod tests {
    use super::{DiscreteRange, DiscreteValue, Error, Normal, RawValue};

    #[test]
    fn range_round_trip_min_zero() {
        const MAX_BOUND: RawValue = RawValue::new(24);
        const CENTER: RawValue = RawValue::new(MAX_BOUND.as_u8() / 2);

        let range = DiscreteRange::new(RawValue::ZERO, MAX_BOUND);

        let normal = range.try_normalize(RawValue::ZERO).unwrap();
        assert_eq!(normal, Normal::MIN);
        assert_eq!(DiscreteValue { normal }.to_raw(range), RawValue::ZERO);

        let normal = range.try_normalize(CENTER).unwrap();
        assert_eq!(normal, Normal::CENTER);
        assert_eq!(DiscreteValue { normal }.to_raw(range), CENTER);

        let normal = range.try_normalize(MAX_BOUND).unwrap();
        assert_eq!(normal, Normal::MAX);
        assert_eq!(DiscreteValue { normal }.to_raw(range), MAX_BOUND);
    }

    #[test]
    fn range_round_trip_min_one() {
        const MIN_BOUND: RawValue = RawValue::new(1);
        const MAX_BOUND: RawValue = RawValue::new(25);
        const CENTER: RawValue = RawValue::new((MAX_BOUND.as_u8() + MIN_BOUND.as_u8()) / 2);

        let range = DiscreteRange::new(MIN_BOUND, MAX_BOUND);

        let normal = range.try_normalize(MIN_BOUND).unwrap();
        assert_eq!(normal, Normal::MIN);
        assert_eq!(DiscreteValue { normal }.to_raw(range), MIN_BOUND);

        let normal = range.try_normalize(CENTER).unwrap();
        assert_eq!(normal, Normal::CENTER);
        assert_eq!(DiscreteValue { normal }.to_raw(range), CENTER);

        let normal = range.try_normalize(MAX_BOUND).unwrap();
        assert_eq!(normal, Normal::MAX);
        assert_eq!(DiscreteValue { normal }.to_raw(range), MAX_BOUND);
    }

    #[test]
    fn out_of_range() {
        const MIN_BOUND: RawValue = RawValue::new(10);
        const MAX_BOUND: RawValue = RawValue::new(20);

        let range = DiscreteRange::new(MIN_BOUND, MAX_BOUND);

        let res = range.try_normalize(RawValue::MIN);
        if let Error::ValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, RawValue::MIN.as_u8());
            assert_eq!(min, MIN_BOUND.as_u8());
            assert_eq!(max, MAX_BOUND.as_u8());
        }

        let res = range.try_normalize(RawValue::MAX);
        if let Error::ValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, RawValue::MAX.as_u8() as u8);
            assert_eq!(min, MIN_BOUND.as_u8());
            assert_eq!(max, MAX_BOUND.as_u8());
        }
    }
}
