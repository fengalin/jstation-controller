use crate::jstation::Error;

/// A Normal (0.0..=1.0) value based on an `f32`.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Normal(f32);

impl Normal {
    pub const MIN: Normal = Normal(0.0);
    pub const CENTER: Normal = Normal(0.5);
    pub const MAX: Normal = Normal(1.0);

    #[inline]
    pub fn as_ratio(self) -> f32 {
        self.0
    }

    /// Tries to build a `Normal` from the provided zero based value and range.
    ///
    /// Returns an `Error` if the `value` is greated than `max`.
    #[inline]
    pub fn try_normalize(value: u8, range: u8) -> Result<Normal, Error> {
        if value > range {
            return Err(Error::ValueOutOfRange {
                value,
                min: 0,
                max: range,
            });
        }

        Ok(Self(value as f32 / range as f32))
    }
}

impl From<Normal> for f32 {
    fn from(normal: Normal) -> f32 {
        normal.as_ratio()
    }
}

impl TryFrom<f32> for Normal {
    type Error = Error;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !(0.0..=1.0).contains(&value) {
            return Err(Error::NormalOutOfRange(value));
        }

        Ok(Normal(value))
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, Normal};

    #[test]
    fn normal() {
        assert_eq!(Normal::try_from(0.0).unwrap(), Normal::MIN);
        assert_eq!(Normal::try_from(0.5).unwrap(), Normal::CENTER);
        assert_eq!(Normal::try_from(1.0).unwrap(), Normal::MAX);

        match Normal::try_from(0.0 - f32::EPSILON).unwrap_err() {
            Error::NormalOutOfRange(val) => assert_eq!(val, 0.0 - f32::EPSILON),
            other => panic!("{other}"),
        }

        match Normal::try_from(Normal::MAX.0 + f32::EPSILON).unwrap_err() {
            Error::NormalOutOfRange(val) => assert_eq!(val, Normal::MAX.0 + f32::EPSILON),
            other => panic!("{other}"),
        }
    }

    #[test]
    fn out_of_range() {
        let res = Normal::try_normalize(11, 10);
        if let Error::ValueOutOfRange { value, min, max } = res.unwrap_err() {
            assert_eq!(value, 11);
            assert_eq!(min, 0);
            assert_eq!(max, 10);
        }
    }
}
