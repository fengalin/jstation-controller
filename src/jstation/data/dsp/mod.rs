use crate::{jstation::Error, midi::CC};

macro_rules! declare_cc_params(
    ( $( $module:ident: $( $cc_param:ident $(,)? )*; )* ) => {
        $(
            mod $module;
            pub use $module::{$( $cc_param, )*};
        )*

        #[derive(Copy, Clone, Debug)]
        pub enum CCParameter {
            $( $(
                $cc_param($cc_param),
            )* )*
        }

        $( $(
            impl From<$cc_param> for CCParameter {
                fn from(cc_param: $cc_param) -> Self {
                    CCParameter::$cc_param(cc_param)
                }
            }
        )* )*

        impl TryFrom<CC> for CCParameter {
            type Error = Error;

            fn try_from(cc: CC) -> Result<Self, Self::Error> {
                use crate::jstation::data::{BoolCCParameter, DiscreteCCParameter};

                match cc.nb {
                    $( $(
                        $cc_param::CC_NB => {
                            Ok(CCParameter::$cc_param($cc_param::from_cc(cc.value)))
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
);

declare_cc_params!(
    amp: AmpModeling, Gain, Treble, Middle, Bass, Level;
    cabinet: Cabinet;
    noise_gate: NoiseGateOn, NoiseGateAttackTime, NoiseGateThreshold;
    utility_settings: DigitalOutLevel;
);
