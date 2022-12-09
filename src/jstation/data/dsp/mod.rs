use crate::{jstation::Error, midi::CC};

pub mod amp;
pub use amp::Amp;

pub mod cabinet;

pub mod noise_gate;
pub use noise_gate::NoiseGate;

pub mod utility_settings;
// `MidiChannel` is not associated with a CC param.
// It is updated as part of the `UtilitySettingsResp` `procedure`.
pub use utility_settings::{MidiChannel, UtilitySettings};

use crate::jstation::data::{BoolCCParameter, DiscreteCCParameter};

macro_rules! declare_params {
    // We could generate $set_param if macro `std::concat_idents` was stable.
    ( $( ($module:ident, $set:ident, $set_param:ident): $( $param:ident $(,)? )*; )* ) => {
        #[derive(Copy, Clone, Debug)]
        pub enum Parameter {
            $( $set($set_param), )*
        }

        $(
            #[derive(Copy, Clone, Debug)]
            pub enum $set_param {
                $( $param($module::$param), )*
            }

            impl From<$set_param> for Parameter {
                fn from(set_param: $set_param) -> Self {
                    Parameter::$set(set_param)
                }
            }

            impl $set_param {
                pub fn to_cc(self) -> CC {
                    match self {
                        $( $set_param::$param(param) => param.to_cc(), )*
                    }
                }
            }

            $(
                impl From<$module::$param> for $set_param {
                    fn from(param: $module::$param) -> Self {
                        $set_param::$param(param)
                    }
                }

                impl From<&$module::$param> for $set_param {
                    fn from(param: &$module::$param) -> Self {
                        $set_param::$param(*param)
                    }
                }

                impl From<&mut $module::$param> for $set_param {
                    fn from(param: &mut $module::$param) -> Self {
                        $set_param::$param(*param)
                    }
                }
            )*
        )*

        impl Parameter {
            pub fn to_cc(self) -> CC {
                match self {
                    $( Parameter::$set(set_param) => set_param.to_cc(), )*
                }
            }
        }

        impl TryFrom<CC> for Parameter {
            type Error = Error;

            fn try_from(cc: CC) -> Result<Self, Self::Error> {
                use crate::jstation::data::{BoolCCParameter, DiscreteCCParameter};

                match cc.nb {
                    $( $(
                        $module::$param::CC_NB => {
                            Ok(Parameter::$set($module::$param::from_cc(cc.value).into()))
                        }
                    )* )*
                    _ => {
                        let err = Error::CCNumber(cc.nb.as_u8());
                        log::warn!("{err}");

                        Err(err)
                    }
                }
            }
        }
    };
}

declare_params!(
    (amp, Amp, AmpParameter): Modeling, Gain, Treble, Middle, Bass, Level;
    (cabinet, Cabinet, CabinetParameter): Type;
    (noise_gate, NoiseGate, NoiseGateParameter): GateOn, AttackTime, Threshold;
    (utility_settings, UtilitySettings, UtilitySettingsParameter): DigitalOutLevel;
);
