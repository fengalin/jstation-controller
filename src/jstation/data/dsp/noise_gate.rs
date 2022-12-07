use crate::{
    jstation::data::{Normal, ParameterNumber, RawValue},
    midi::CCNumber,
};

bool_parameter!(NoiseGateOn {
    const DEFAULT = false,
    const PARAMETER_NB = ParameterNumber::new(16),
    const CC_NB = CCNumber::new(41),
});

discrete_parameter!(NoiseGateAttackTime {
    const DEFAULT = Normal::MIN,
    const MAX_RAW = RawValue::new(10),
    const PARAMETER_NB = ParameterNumber::new(17),
    const CC_NB = CCNumber::new(42),
});

discrete_parameter!(NoiseGateThreshold {
    const DEFAULT = Normal::MIN,
    const MIN_RAW = RawValue::new(1),
    const MAX_RAW = RawValue::new(99),
    const PARAMETER_NB = ParameterNumber::new(18),
    const CC_NB = CCNumber::new(43),
});
