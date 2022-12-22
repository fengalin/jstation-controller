use crate::jstation::{
    data::{Normal, ParameterSetter, RawValue},
    Error,
};
use crate::midi;

pub trait DiscreteParameter: ParameterSetter<Parameter = Self> + Clone + Copy {
    fn param_name(self) -> &'static str;

    fn normal_default(self) -> Option<Normal>;

    fn normal(self) -> Option<Normal>;

    fn raw_value(self) -> Option<RawValue>;

    /// Resets the parameter to its default value.
    fn reset(&mut self) -> Option<Self>;

    fn is_active(self) -> bool {
        true
    }
}

/// An inclusive `DiscreteRange` of discrete [`Value`]s between 0 and `max`.
///
/// [`Value`]: ../raw/struct.Value.html
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiscreteRange {
    min: u8,
    delta: u8,
}

impl DiscreteRange {
    const MAX_CC_U32: u32 = midi::CCValue::MAX.as_u8() as u32;
    const SCALE: u32 = 0x1_0000;
    const ROUNDING: u32 = Self::SCALE / 2;
    const MAX_CC_SCALED: u32 = Self::MAX_CC_U32 * Self::SCALE;

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

    #[inline]
    pub fn check(self, value: RawValue) -> Result<RawValue, Error> {
        if !(self.min..=(self.min + self.delta)).contains(&value.as_u8()) {
            return Err(self.out_of_range_error(value));
        }

        Ok(value)
    }

    #[inline]
    fn out_of_range_error(self, value: impl Into<u8>) -> Error {
        Error::ValueOutOfRange {
            value: value.into(),
            min: self.min,
            max: self.delta + self.min,
        }
    }

    #[inline]
    fn zero_based(self, value: RawValue) -> Result<u8, Error> {
        value
            .as_u8()
            .checked_sub(self.min)
            .ok_or_else(|| self.out_of_range_error(value))
    }

    /// Tries to build a `Normal` from the provided `value`.
    ///
    /// The value must fit in this `DiscreteRange`.
    pub fn try_normalize(self, value: RawValue) -> Result<Normal, Error> {
        let zero_based_value = self.zero_based(value)?;

        Normal::try_normalize(zero_based_value, self.delta)
            .map_err(|_| self.out_of_range_error(value))
    }

    pub fn normal_to_raw(self, normal: Normal) -> RawValue {
        let zero_based_value = (self.delta as f32 * normal.as_ratio()).round() as u8;

        RawValue::new(zero_based_value + self.min)
    }

    pub fn try_ccize(self, value: RawValue) -> Result<midi::CCValue, Error> {
        let zero_based_value = self.zero_based(value)?;
        if zero_based_value > self.delta {
            return Err(self.out_of_range_error(value));
        }

        let cc_value = (zero_based_value as u32 * Self::MAX_CC_SCALED / (self.delta as u32)
            + Self::ROUNDING)
            / Self::SCALE;

        Ok(midi::CCValue::new_clipped(cc_value as u8))
    }

    pub fn cc_to_raw(self, cc_value: midi::CCValue) -> RawValue {
        // Get the zero based value in this range.
        // round up otherwise the decimal value is simply truncated.
        let zero_based = (cc_value.as_u8() as u32 * (self.delta as u32) * Self::SCALE
            / Self::MAX_CC_U32
            + Self::ROUNDING)
            / Self::SCALE;

        RawValue::new(zero_based as u8 + self.min)
    }

    pub fn to_cents(self, value: RawValue) -> Result<u8, Error> {
        let zero_based_value = self.zero_based(value)?;
        if zero_based_value > self.delta {
            return Err(self.out_of_range_error(value));
        }

        let cents = (1000 * (zero_based_value as u32) / (self.delta as u32) + 5) / 10;

        Ok(cents as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::{midi::CCValue, DiscreteRange, Error, Normal, RawValue};

    #[test]
    fn range_round_trip_min_zero() {
        const MAX_BOUND: RawValue = RawValue::new(24);
        const CENTER: RawValue = RawValue::new(MAX_BOUND.as_u8() / 2);

        let range = DiscreteRange::new(RawValue::ZERO, MAX_BOUND);

        let normal = range.try_normalize(RawValue::ZERO).unwrap();
        assert_eq!(normal, Normal::MIN);
        assert_eq!(range.normal_to_raw(normal), RawValue::ZERO);

        let normal = range.try_normalize(CENTER).unwrap();
        assert_eq!(normal, Normal::CENTER);
        assert_eq!(range.normal_to_raw(normal), CENTER);

        let normal = range.try_normalize(MAX_BOUND).unwrap();
        assert_eq!(normal, Normal::MAX);
        assert_eq!(range.normal_to_raw(normal), MAX_BOUND);
    }

    #[test]
    fn range_round_trip_min_one() {
        const MIN_BOUND: RawValue = RawValue::new(1);
        const MAX_BOUND: RawValue = RawValue::new(25);
        const CENTER: RawValue = RawValue::new((MAX_BOUND.as_u8() + MIN_BOUND.as_u8()) / 2);

        let range = DiscreteRange::new(MIN_BOUND, MAX_BOUND);

        let normal = range.try_normalize(MIN_BOUND).unwrap();
        assert_eq!(normal, Normal::MIN);
        assert_eq!(range.normal_to_raw(normal), MIN_BOUND);

        let normal = range.try_normalize(CENTER).unwrap();
        assert_eq!(normal, Normal::CENTER);
        assert_eq!(range.normal_to_raw(normal), CENTER);

        let normal = range.try_normalize(MAX_BOUND).unwrap();
        assert_eq!(normal, Normal::MAX);
        assert_eq!(range.normal_to_raw(normal), MAX_BOUND);
    }

    #[test]
    fn out_of_range() {
        const OO_MIN_BOUND: RawValue = RawValue::new(9);
        const MIN_BOUND: RawValue = RawValue::new(10);
        const MAX_BOUND: RawValue = RawValue::new(20);
        const OO_MAX_BOUND: RawValue = RawValue::new(21);

        const RANGE: DiscreteRange = DiscreteRange::new(MIN_BOUND, MAX_BOUND);

        let res = RANGE.try_normalize(OO_MIN_BOUND);
        if let Error::ValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, OO_MIN_BOUND.as_u8());
            assert_eq!(min, MIN_BOUND.as_u8());
            assert_eq!(max, MAX_BOUND.as_u8());
        }

        let res = RANGE.try_normalize(OO_MAX_BOUND);
        if let Error::ValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, OO_MAX_BOUND.as_u8() as u8);
            assert_eq!(min, MIN_BOUND.as_u8());
            assert_eq!(max, MAX_BOUND.as_u8());
        }

        let res = RANGE.try_ccize(OO_MIN_BOUND);
        if let Error::ValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, OO_MIN_BOUND.as_u8());
            assert_eq!(min, MIN_BOUND.as_u8());
            assert_eq!(max, MAX_BOUND.as_u8());
        }

        let res = RANGE.try_ccize(OO_MAX_BOUND);
        if let Error::ValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, OO_MAX_BOUND.as_u8() as u8);
            assert_eq!(min, MIN_BOUND.as_u8());
            assert_eq!(max, MAX_BOUND.as_u8());
        }

        let res = RANGE.to_cents(OO_MIN_BOUND);
        if let Error::ValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, OO_MIN_BOUND.as_u8());
            assert_eq!(min, MIN_BOUND.as_u8());
            assert_eq!(max, MAX_BOUND.as_u8());
        }

        let res = RANGE.to_cents(OO_MAX_BOUND);
        if let Error::ValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, OO_MAX_BOUND.as_u8() as u8);
            assert_eq!(min, MIN_BOUND.as_u8());
            assert_eq!(max, MAX_BOUND.as_u8());
        }
    }

    #[test]
    fn cc_to_raw() {
        const MIN_BOUND: RawValue = RawValue::new(10);
        const CENTER_BOUND: RawValue = RawValue::new(15);
        const MAX_BOUND: RawValue = RawValue::new(20);

        const RANGE: DiscreteRange = DiscreteRange::new(MIN_BOUND, MAX_BOUND);

        assert_eq!(RANGE.cc_to_raw(CCValue::ZERO), MIN_BOUND);
        assert_eq!(
            RANGE.cc_to_raw(CCValue::new_clipped(CCValue::MAX.as_u8() / 2 + 1)),
            CENTER_BOUND,
        );
        assert_eq!(RANGE.cc_to_raw(CCValue::MAX), MAX_BOUND);
    }

    #[test]
    fn try_ccize() {
        const MIN_BOUND: RawValue = RawValue::new(10);
        const CENTER_BOUND: RawValue = RawValue::new(15);
        const MAX_BOUND: RawValue = RawValue::new(20);

        const RANGE: DiscreteRange = DiscreteRange::new(MIN_BOUND, MAX_BOUND);

        assert_eq!(RANGE.try_ccize(MIN_BOUND).unwrap(), CCValue::ZERO);
        assert_eq!(
            RANGE.try_ccize(CENTER_BOUND).unwrap(),
            CCValue::new_clipped(CCValue::MAX.as_u8() / 2 + 1),
        );
        assert_eq!(RANGE.try_ccize(MAX_BOUND).unwrap(), CCValue::MAX);
    }

    #[test]
    fn to_cents() {
        const MIN_BOUND: RawValue = RawValue::new(10);
        const CENTER_BOUND: RawValue = RawValue::new(15);
        const MAX_BOUND: RawValue = RawValue::new(20);

        const RANGE: DiscreteRange = DiscreteRange::new(MIN_BOUND, MAX_BOUND);

        assert_eq!(RANGE.to_cents(MIN_BOUND).unwrap(), 0);
        assert_eq!(RANGE.to_cents(CENTER_BOUND).unwrap(), 50);
        assert_eq!(RANGE.to_cents(MAX_BOUND).unwrap(), 100);
    }
}
