use std::fmt;

use crate::{
    jstation::{data::DiscreteParameter, CCParameter},
    midi,
};

pub mod amp;
pub use amp::Amp;

pub mod cabinet;
pub use cabinet::Cabinet;

pub mod compressor;
pub use compressor::Compressor;

pub mod noise_gate;
pub use noise_gate::NoiseGate;

pub mod utility_settings;
pub use utility_settings::UtilitySettings;

fn fmt_percent(param: impl DiscreteParameter, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_fmt(format_args!(
        "{:3.0}",
        100.0 * param.into().normal().as_f32()
    ))
}

fn fmt_bipolar_normal(param: impl DiscreteParameter, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let bipolar = 2.0 * param.into().normal().as_f32() - 1.0;
    f.write_fmt(format_args!("{:0.2}", bipolar))
}

#[derive(Clone, Copy, Debug)]
pub enum Parameter {
    Amp(amp::Parameter),
    Cabinet(cabinet::Parameter),
    Compressor(compressor::Parameter),
    NoiseGate(noise_gate::Parameter),
    UtilitySettings(utility_settings::Parameter),
}

// FIXME impl ParameterSetter for a Device struct using Parameter?

impl CCParameter for Parameter {
    fn from_cc(cc: midi::CC) -> Option<Self> {
        // FIXME not ideal: we might be able to match on cc nb ranges instead

        let mut param = amp::Parameter::from_cc(cc).map(Parameter::Amp);
        if param.is_some() {
            return param;
        }
        param = cabinet::Parameter::from_cc(cc).map(Parameter::Cabinet);
        if param.is_some() {
            return param;
        }
        param = compressor::Parameter::from_cc(cc).map(Parameter::Compressor);
        if param.is_some() {
            return param;
        }
        param = noise_gate::Parameter::from_cc(cc).map(Parameter::NoiseGate);
        if param.is_some() {
            return param;
        }
        param = utility_settings::Parameter::from_cc(cc).map(Parameter::UtilitySettings);
        if param.is_some() {
            return param;
        }

        None
    }

    fn to_cc(self) -> Option<midi::CC> {
        match self {
            Parameter::Amp(param) => param.to_cc(),
            Parameter::Cabinet(param) => param.to_cc(),
            Parameter::Compressor(param) => param.to_cc(),
            Parameter::NoiseGate(param) => param.to_cc(),
            Parameter::UtilitySettings(param) => param.to_cc(),
        }
    }
}
