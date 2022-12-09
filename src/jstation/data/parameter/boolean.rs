use crate::{
    jstation::data::{ParameterNumber, RawParameter, RawValue, ValueStatus},
    midi::{CCNumber, CCValue, CC},
};

pub trait BoolParameter:
    From<bool> + Into<bool> + PartialEq + Copy + Clone + std::fmt::Debug + Sized
{
    const DEFAULT: bool;

    fn from_raw(raw: RawValue) -> Self {
        (raw.as_u8() == 0).into()
    }

    fn to_raw_value(self) -> RawValue {
        RawValue::new(if self.into() { 0 } else { u8::MAX })
    }

    fn is_active(self) -> bool {
        self.into()
    }

    /// Resets the parameter to its default value.
    fn reset(&mut self) -> ValueStatus {
        self.set(Self::DEFAULT.into())
    }

    /// Sets the value if it is different than current.
    fn set(&mut self, new: Self) -> ValueStatus {
        if new == *self {
            return ValueStatus::Unchanged;
        }

        *self = new;

        ValueStatus::Changed
    }
}

/// A `BoolParameter`, view as a raw `Parameter`, e.g. part of a `Program` `data`.
pub trait BoolRawParameter: BoolParameter {
    const PARAMETER_NB: ParameterNumber;

    fn to_raw_parameter(self) -> RawParameter {
        RawParameter::new(Self::PARAMETER_NB, self.to_raw_value())
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
    ( $param:ident { const DEFAULT = $default:expr $(,)? } ) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $param(bool);

        impl crate::jstation::data::BoolParameter for $param {
            const DEFAULT: bool = $default;
        }

        impl Default for $param {
            fn default() -> Self {
                $param($default)
            }
        }

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
    };

    ( #[derive(Display)] $param:ident {
        const DEFAULT = $default:expr $(,)?
    } ) => {
        bool_parameter!($param { const DEFAULT = $default });

        bool_parameter!(#[derive(Display)] $param);
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

    ( #[derive(Display)] $param:ident {
        const DEFAULT = $default:expr,
        const PARAMETER_NB = $param_nb:expr $(,)?
    } ) => {
        bool_parameter!($param {
            const DEFAULT = $default,
            const PARAMETER_NB = $param_nb,
        });

        bool_parameter!(#[derive(Display)] $param);
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

    ( #[derive(Display)] $param:ident {
        const DEFAULT = $default:expr,
        const CC_NB = $cc_nb:expr $(,)?
    } ) => {
        bool_parameter!($param {
            const DEFAULT = $default,
            const CC_NB = $cc_nb,
        });

        bool_parameter!(#[derive(Display)] $param);
    };

    ( $param:ident {
        const DEFAULT = $default:expr,
        const PARAMETER_NB = $param_nb:expr,
        const CC_NB = $cc_nb:expr $(,)?
    } ) => {
        bool_parameter!($param {
            const DEFAULT = $default,
            const PARAMETER_NB = $param_nb,
        });

        impl crate::jstation::data::BoolCCParameter for $param {
            const CC_NB: crate::jstation::midi::CCNumber = $cc_nb;
        }
    };

    ( #[derive(Display)] $param:ident {
        const DEFAULT = $default:expr,
        const PARAMETER_NB = $param_nb:expr,
        const CC_NB = $cc_nb:expr $(,)?
    } ) => {
        bool_parameter!($param {
            const DEFAULT = $default,
            const PARAMETER_NB = $param_nb,
            const CC_NB = $cc_nb,
        });

        bool_parameter!(#[derive(Display)] $param);
    };

    ( #[derive(Display)] $param:ident ) => {
        impl std::fmt::Display for $param {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use crate::jstation::data::BoolParameter;
                if self.is_active() {
                    f.write_str("on")
                } else {
                    f.write_str("off")
                }
            }
        }
    };
}
