use crate::{jstation::Error, midi::CCValue};

/// A Normal value baed on an `u8`.
///
/// Internal computation uses an `u32` so as to reduce precision loss.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, PartialOrd)]
pub struct Normal(u8);

impl Normal {
    pub const MIN: Normal = Normal(0);
    pub const CENTER: Normal = Normal(0x80);
    pub const MAX: Normal = Normal(0xff);

    const MAX_U32: u32 = Self::MAX.0 as u32;
    const SCALE: u32 = 0x1000;
    const ROUNDING: u32 = Self::SCALE / 2;
    const MAX_SCALED: u32 = Self::MAX_U32 * Self::SCALE;

    const MAX_F32: f32 = Self::MAX.0 as f32;
    const MAX_RECIP_F32: f32 = 1.0 / (Self::MAX.0 as f32);

    pub const fn as_u8(self) -> u8 {
        self.0
    }

    pub fn as_ratio(self) -> f32 {
        self.0 as f32 * Normal::MAX_RECIP_F32
    }

    pub fn as_cents(self) -> u16 {
        ((self.0 as u32 * 100 * Self::SCALE + Self::ROUNDING) / Self::MAX_SCALED) as u16
    }

    // FIXME for all the function below, range must NOT exceed RawValue::MAX...

    /// Tries to build a `Normal` from the provided zero based value and range.
    ///
    /// Returns an `Error` if the `value` is greated than `max`.
    pub fn try_normalize(value: u8, range: u8) -> Result<Normal, Error> {
        if value > range {
            return Err(Error::ValueOutOfRange {
                value,
                min: 0,
                max: range,
            });
        }

        let value = value as u32;
        let range = range as u32;

        Ok(Self::normalize_priv(value, range))
    }

    fn normalize_priv(value: u32, range: u32) -> Normal {
        Normal(((value * Self::MAX_SCALED / range + Self::ROUNDING) / Self::SCALE) as u8)
    }

    /// Maps this `Normal` to the provided range.
    pub fn map_to(self, range: u8) -> u8 {
        let range = range as u32;

        self.map_to_priv(range) as u8
    }

    #[inline(always)]
    fn map_to_priv(self, range: u32) -> u32 {
        // round up otherwise the decimal value is simply truncated.
        (self.0 as u32 * range * Self::SCALE / Self::MAX_U32 + Self::ROUNDING) / Self::SCALE
    }

    /// Snaps this `Normal` to the quantification of the provided range.
    pub fn snap_to(self, range: u8) -> Normal {
        let range = range as u32;
        let value = self.map_to_priv(range);

        Self::normalize_priv(value, range)
    }
}

impl From<Normal> for CCValue {
    fn from(normal: Normal) -> Self {
        unsafe {
            // Safety: CC Value is 0 based and we know how to map this normal
            // to the CCValue's range.
            std::mem::transmute(normal.map_to(CCValue::MAX.as_u8()))
        }
    }
}

impl From<CCValue> for Normal {
    fn from(value: CCValue) -> Self {
        Self::normalize_priv(value.as_u8() as u32, CCValue::MAX.as_u8() as u32)
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

        Ok(Normal((value * Self::MAX_F32).ceil() as u8))
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

        match Normal::try_from(0.0 - f32::EPSILON).unwrap_err() {
            Error::NormalOutOfRange(val) => assert_eq!(val, 0.0 - f32::EPSILON),
            other => panic!("{other}"),
        }

        match Normal::try_from(Normal::MAX_F32 + f32::EPSILON).unwrap_err() {
            Error::NormalOutOfRange(val) => assert_eq!(val, Normal::MAX_F32 + f32::EPSILON),
            other => panic!("{other}"),
        }
    }

    #[test]
    fn round_trip() {
        const FORTY_NINE: u8 = 49;
        const RANGE: u8 = 97;

        let normal = Normal::try_normalize(0, RANGE).unwrap();
        assert_eq!(normal, Normal::MIN);
        assert_eq!(normal.map_to(RANGE), 0);

        let normal = Normal::try_normalize(FORTY_NINE, RANGE).unwrap();
        assert_eq!(normal.map_to(RANGE), FORTY_NINE);

        let normal = Normal::try_normalize(RANGE, RANGE).unwrap();
        assert_eq!(normal, Normal::MAX);
        assert_eq!(normal.map_to(RANGE), RANGE);
    }

    #[test]
    fn snap_to() {
        const R23: u8 = 23;
        const R24: u8 = 24;
        const R25: u8 = 25;

        assert_eq!(Normal::CENTER.snap_to(R23).map_to(R23), 12);
        assert_eq!(Normal::CENTER.snap_to(R24).map_to(R24), 12);
        assert_eq!(Normal::CENTER.snap_to(R25).map_to(R25), 13);

        let normal_third = Normal::try_from(1.0 / 3.0).unwrap();
        assert_eq!(normal_third.snap_to(R23).map_to(R23), 8);
        assert_eq!(normal_third.snap_to(R24).map_to(R24), 8);
        assert_eq!(normal_third.snap_to(R25).map_to(R25), 8);

        assert_eq!(Normal::MAX.snap_to(R24).map_to(R24), 24);
        assert_eq!(Normal::MAX.snap_to(R25).map_to(R25), 25);
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

    #[test]
    fn cc_value() {
        assert_eq!(CCValue::from(Normal::MIN), CCValue::ZERO);
        assert_eq!(CCValue::from(Normal::MAX), CCValue::MAX);

        assert_eq!(Normal::from(CCValue::ZERO), Normal::MIN);
        assert_eq!(Normal::from(CCValue::MAX), Normal::MAX);
    }
}
