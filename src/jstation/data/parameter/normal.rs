use crate::{jstation::Error, midi::CCValue};

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Normal(f32);

impl Normal {
    pub const MIN: Normal = Normal(0.0);
    pub const CENTER: Normal = Normal(0.5);
    pub const MAX: Normal = Normal(1.0);

    const MAX_CC_VALUE: f32 = CCValue::MAX.as_u8() as f32;

    pub const fn as_f32(self) -> f32 {
        self.0
    }

    pub fn into_cc_value(self) -> CCValue {
        // FIXME we could impl `CCValue::new_unchecked` instead.
        CCValue::new_clipped((self.0 * Self::MAX_CC_VALUE) as u8)
    }

    /// Builds a `Normal` from the provided value, whithout checking it.
    ///
    /// # Safety
    ///
    /// The value must be in the range `(0.0..=1.0)`.
    pub unsafe fn new_unchecked(normal: f32) -> Self {
        Normal(normal)
    }
}

impl From<Normal> for CCValue {
    fn from(normal: Normal) -> Self {
        normal.into_cc_value()
    }
}

impl From<CCValue> for Normal {
    fn from(value: CCValue) -> Self {
        Normal(value.as_u8() as f32 / Normal::MAX_CC_VALUE)
    }
}

impl From<Normal> for f32 {
    fn from(normal: Normal) -> Self {
        normal.0
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
    use super::{CCValue, Error, Normal};

    #[test]
    fn normal() {
        assert_eq!(Normal::try_from(0.0).unwrap(), Normal::MIN);
        assert_eq!(Normal::try_from(0.5).unwrap(), Normal::CENTER);
        assert_eq!(Normal::try_from(1.0).unwrap(), Normal::MAX);

        match Normal::try_from(Normal::MIN.0 - f32::EPSILON).unwrap_err() {
            Error::NormalOutOfRange(val) => assert_eq!(val, Normal::MIN.0 - f32::EPSILON),
            other => panic!("{other}"),
        }

        match Normal::try_from(Normal::MAX.0 + f32::EPSILON).unwrap_err() {
            Error::NormalOutOfRange(val) => assert_eq!(val, Normal::MAX.0 + f32::EPSILON),
            other => panic!("{other}"),
        }
    }

    #[test]
    fn cc_value() {
        assert_eq!(CCValue::from(Normal::MIN), CCValue::ZERO);
        assert_eq!(CCValue::from(Normal::MAX), CCValue::MAX);

        assert_eq!(Normal::from(CCValue::ZERO), Normal::MIN);
        assert_eq!(Normal::from(CCValue::MAX), Normal::MAX);
    }
}
