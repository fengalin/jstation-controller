use crate::{
    jstation::data::{Normal, RawValue},
    midi::CCNumber,
};

discrete_parameter!(DigitalOutLevel {
    const DEFAULT = Normal::MIN,
    const MAX_RAW = RawValue::new(24),
    const CC_NB = CCNumber::new(14),
});
