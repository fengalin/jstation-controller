use crate::{
    jstation::data::{ParameterNumber, RawParameter, RawValue},
    midi::{CCNumber, CCValue, CC},
};

pub trait BoolParameter: From<bool> + Into<bool> + Copy + Clone + std::fmt::Debug + Sized {
    const DEFAULT: bool;

    /// Resets the parameter to its default value.
    fn reset(&mut self) {
        *self = Self::DEFAULT.into();
    }

    fn value(self) -> bool {
        self.into()
    }

    fn set(&mut self, value: bool) {
        *self = value.into();
    }
}

/// A `BoolParameter`, view as a raw `Parameter`, e.g. part of a `Program` `data`.
pub trait BoolRawParameter: BoolParameter {
    const PARAMETER_NB: ParameterNumber;

    fn from_raw(raw: RawValue) -> Self {
        (raw.as_u8() == 0).into()
    }

    fn to_raw(self) -> RawParameter {
        let value = RawValue::new(if self.into() { 0 } else { u8::MAX });

        RawParameter::new(Self::PARAMETER_NB, value)
    }
}

/// A `BoolParameter`, view as a CC `Parameter`, e.g. part of received as MIDI `CC` message.
pub trait BoolCCParameter: BoolParameter {
    const CC_NB: CCNumber;

    fn from_cc(cc_val: CCValue) -> Self {
        const CC_TRUE_THRESHOLD: u8 = 0x40;

        (cc_val.as_u8() >= CC_TRUE_THRESHOLD).into()
    }

    fn to_cc(self) -> CC {
        let value = if self.into() {
            CCValue::MAX
        } else {
            CCValue::ZERO
        };

        CC::new(Self::CC_NB, value)
    }
}

macro_rules! bool_parameter {
    ($param:ident { const DEFAULT = $default:expr $(,)? } ) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $param(bool);

        impl From<bool> for $param {
            fn from(value: bool) -> Self {
                $param(value)
            }
        }

        impl From<$param> for bool {
            fn from(value: $param) -> Self {
                value.0
            }
        }

        impl crate::jstation::data::BoolParameter for $param {
            const DEFAULT: bool = $default;
        }
    };

    ( $param:ident {
        const DEFAULT = $default:expr,
        const PARAMETER_NB = $param_nb:expr $(,)?
    } ) => {
        bool_parameter!($param { const DEFAULT = $default });

        impl crate::jstation::data::BoolRawParameter for $param {
            const PARAMETER_NB: crate::jstation::data::ParameterNumber = $param_nb;
        }
    };

    ( $param:ident {
        const DEFAULT = $default:expr,
        const CC_NB = $cc_nb:expr $(,)?
    } ) => {
        bool_parameter!($param { const DEFAULT = $default });

        impl crate::jstation::data::BoolCCParameter for $param {
            const CC_NB: crate::midi::CCNumber = $cc_nb;
        }
    };

    ( $param:ident {
        const DEFAULT = $default:expr,
        const PARAMETER_NB = $param_nb:expr,
        const CC_NB = $cc_nb:expr $(,)?
    } ) => {
        bool_parameter!($param {
            const DEFAULT = $default,
            const PARAMETER_NB = $param_nb
        });

        impl crate::jstation::data::BoolCCParameter for $param {
            const CC_NB: crate::jstation::midi::CCNumber = $cc_nb;
        }
    };
}
