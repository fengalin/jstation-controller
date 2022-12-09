use crate::jstation::{
    data::{Normal, ParameterNumber, RawParameter, RawValue},
    Error,
};

pub trait DiscreteParameter:
    From<DiscreteValue> + Into<DiscreteValue> + PartialEq + Copy + Clone + std::fmt::Debug + Sized
{
    const NAME: &'static str;
    const DEFAULT: Normal;
    const MIN_RAW: RawValue;
    const MAX_RAW: RawValue;
    const RANGE: crate::jstation::data::DiscreteRange;

    fn from_snapped(normal: Normal) -> Self {
        DiscreteValue::new(normal, Self::RANGE).into()
    }

    fn try_from_raw(raw: RawValue) -> Result<Self, Error> {
        let value = DiscreteValue::try_from_raw(raw, Self::RANGE)
            .map_err(|err| Error::with_context(Self::NAME, err))?;

        Ok(value.into())
    }

    fn to_raw_value(self) -> RawValue {
        self.into().get_raw(Self::RANGE)
    }

    fn normal(self) -> Normal {
        self.into().normal()
    }

    /// Resets the parameter to its default value.
    fn reset(&mut self) -> Option<Self> {
        self.set(Self::from_snapped(Self::DEFAULT))
    }

    /// Sets the value if it is different than current.
    ///
    /// Returns the new value if it as changed.
    fn set(&mut self, new: Self) -> Option<Self> {
        if new == *self {
            return None;
        }

        *self = new;

        Some(new)
    }
}

// FIXME might want to define a generic RawParameter similar to CCParameter
/// A `DiscreteParameter`, view as a raw `Parameter`, e.g. part of a `Program` `data`.
pub trait DiscreteRawParameter: DiscreteParameter {
    const PARAMETER_NB: ParameterNumber;

    fn to_raw_parameter(self) -> RawParameter {
        RawParameter::new(Self::PARAMETER_NB, self.to_raw_value())
    }
}

// A discrete value which is guaranteed to be snapped to the provided [`DiscreteRange`].
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
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

    pub fn get_raw(&self, range: DiscreteRange) -> RawValue {
        ((self.normal.as_f32() * range.zero_based_max) as u8 + range.min).into()
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
    zero_based_max: f32,
    // Note: in a previous implementation, I used a `max_recip` to avoid float divisions
    // in some functions, thus allegedly saving a few cycles. However, since float operations
    // can't be used in `const` functions, I had to use a `once_cell` in the parameters and
    // return the `DiscreteRange` lazily, which adds some cycles too and cache misses potentialy.
    // I ended up deciding it wasn't worth the complexity.
}

impl DiscreteRange {
    /// Builds a new `DiscreteRange` from the provided value.
    ///
    /// # Panic
    ///
    /// Panics if `min` < `max`.
    pub const fn new(min: RawValue, max: RawValue) -> Self {
        let min = min.as_u8();
        let max = max.as_u8();

        assert!(min < max);

        DiscreteRange {
            min,
            zero_based_max: (max - min) as f32,
        }
    }

    fn out_of_range_error(self, value: RawValue) -> Error {
        Error::ParameterRawValueOutOfRange {
            value,
            min: self.min.into(),
            max: (self.zero_based_max as u8 + self.min).into(),
        }
    }

    /// Tries to build a `Normal` from the provided `value`.
    fn try_normalize(self, value: RawValue) -> Result<Normal, Error> {
        let zero_based_value = value
            .as_u8()
            .checked_sub(self.min)
            .ok_or_else(|| self.out_of_range_error(value))? as f32;

        let normal = Normal::try_from(zero_based_value / self.zero_based_max)
            .map_err(|_| self.out_of_range_error(value))?;

        Ok(self.snap(normal))
    }

    /// Returns the provided `Normal` after snapping it to this `DiscreteRange`.
    pub fn snap(&self, normal: Normal) -> Normal {
        unsafe {
            // Safety: `self.zero_based_max` is ensured to be a positive integral since
            // the only way to build `DiscreteRange` is via `DiscreteRange::new` which uses
            // a `RawValue` which is guaranteed to be built from an `u8`.
            //
            // The expression `normal_snapped` is expected to be in the range `(0.0..=1.0)`,
            // thanks to the `(0.0..=1.0)` guarantee for `normal`.
            // `(normal.as_f32() * self.zero_based_max).round()` equals at most
            // `self.zero_based_max`, so `normal_snapped` can't be greater than `1.0`.

            let normal_snapped =
                (normal.as_f32() * self.zero_based_max).round() / self.zero_based_max;
            Normal::new_unchecked(normal_snapped)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DiscreteRange, DiscreteValue, Error, Normal, RawValue};

    #[test]
    fn range_round_trip_min_zero() {
        const MAX: RawValue = RawValue::new(24);
        const CENTER: RawValue = RawValue::new(MAX.as_u8() / 2);
        const BEYOND_MAX: RawValue = RawValue::new(MAX.as_u8() + 1);

        let range = DiscreteRange::new(RawValue::ZERO, MAX);

        let normal = range.try_normalize(RawValue::ZERO).unwrap();
        assert_eq!(normal.as_f32(), 0.0);
        assert_eq!(DiscreteValue { normal }.get_raw(range), RawValue::ZERO);

        let normal = range.try_normalize(CENTER).unwrap();
        assert_eq!(normal.as_f32(), 0.5);
        assert_eq!(DiscreteValue { normal }.get_raw(range), CENTER);

        let normal = range.try_normalize(MAX).unwrap();
        assert_eq!(normal.as_f32(), 1.0);
        assert_eq!(DiscreteValue { normal }.get_raw(range), MAX);

        let res = range.try_normalize(BEYOND_MAX);
        if let Error::ParameterRawValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, BEYOND_MAX);
            assert_eq!(min, RawValue::ZERO);
            assert_eq!(max, MAX);
        }
    }

    #[test]
    fn range_round_trip_min_one() {
        const MIN: RawValue = RawValue::new(1);
        const MAX: RawValue = RawValue::new(25);
        const CENTER: RawValue = RawValue::new((MAX.as_u8() + MIN.as_u8()) / 2);
        const BEYOND_MAX: RawValue = RawValue::new(MAX.as_u8() + 1);

        let range = DiscreteRange::new(MIN, MAX);

        let normal = range.try_normalize(MIN).unwrap();
        assert_eq!(normal.as_f32(), 0.0);
        assert_eq!(DiscreteValue { normal }.get_raw(range), MIN);

        let normal = range.try_normalize(CENTER).unwrap();
        assert_eq!(normal.as_f32(), 0.5);
        assert_eq!(DiscreteValue { normal }.get_raw(range), CENTER);

        let normal = range.try_normalize(MAX).unwrap();
        assert_eq!(normal.as_f32(), 1.0);
        assert_eq!(DiscreteValue { normal }.get_raw(range), MAX);

        let res = range.try_normalize(RawValue::ZERO);
        if let Error::ParameterRawValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, RawValue::ZERO);
            assert_eq!(min, MIN);
            assert_eq!(max, MAX);
        }

        let res = range.try_normalize(BEYOND_MAX);
        if let Error::ParameterRawValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, BEYOND_MAX);
            assert_eq!(min, MIN);
            assert_eq!(max, MAX);
        }
    }

    #[test]
    fn snap() {
        let r23 = DiscreteRange::new(RawValue::ZERO, 23.into());
        let r24 = DiscreteRange::new(RawValue::ZERO, 24.into());
        let r25 = DiscreteRange::new(RawValue::ZERO, 25.into());

        assert_eq!(r23.snap(Normal::CENTER).as_f32() * 23.0, 12.0);
        assert_eq!(r24.snap(Normal::CENTER).as_f32() * 24.0, 12.0);
        assert_eq!(r25.snap(Normal::CENTER).as_f32() * 25.0, 13.0);

        let normal_third = Normal::try_from(1.0 / 3.0).unwrap();
        assert_eq!(r23.snap(normal_third).as_f32() * 23.0, 8.0);
        assert_eq!(r24.snap(normal_third).as_f32() * 24.0, 8.0);
        assert_eq!(r25.snap(normal_third).as_f32() * 25.0, 8.0);

        assert_eq!(r24.snap(Normal::MAX).as_f32() * 24.0, 24.0);
        assert_eq!(r25.snap(Normal::MAX).as_f32() * 25.0, 25.0);
    }
}
