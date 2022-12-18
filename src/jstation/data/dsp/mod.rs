use std::fmt;

use crate::{
    jstation::{
        data::{CCParameter, CCParameterSetter, DiscreteParameter, ParameterSetter},
        Error,
    },
    midi,
};

pub mod amp;
pub use amp::Amp;

pub mod cabinet;
pub use cabinet::Cabinet;

pub mod compressor;
pub use compressor::Compressor;

pub mod effect;
pub use effect::Effect;

pub mod noise_gate;
pub use noise_gate::NoiseGate;

pub mod utility_settings;
pub use utility_settings::UtilitySettings;

#[derive(Debug, Default)]
pub struct Dsp {
    pub amp: amp::Amp,
    pub cabinet: cabinet::Cabinet,
    pub compressor: compressor::Compressor,
    pub effect: effect::Effect,
    pub noise_gate: noise_gate::NoiseGate,
    pub utility_settings: utility_settings::UtilitySettings,
}

#[derive(Clone, Copy, Debug)]
pub enum Parameter {
    Amp(amp::Parameter),
    Cabinet(cabinet::Parameter),
    Compressor(compressor::Parameter),
    Effect(effect::Parameter),
    NoiseGate(noise_gate::Parameter),
    UtilitySettings(utility_settings::Parameter),
}

impl ParameterSetter for Dsp {
    type Parameter = Parameter;

    fn set(&mut self, new: Parameter) -> Option<Parameter> {
        use Parameter::*;
        match new {
            Amp(param) => self.amp.set(param).map(Parameter::from),
            Cabinet(param) => self.cabinet.set(param).map(Parameter::from),
            Compressor(param) => self.compressor.set(param).map(Parameter::from),
            Effect(param) => self.effect.set(param).map(Parameter::from),
            NoiseGate(param) => self.noise_gate.set(param).map(Parameter::from),
            UtilitySettings(param) => self.utility_settings.set(param).map(Parameter::from),
        }
    }
}

impl CCParameter for Parameter {
    fn to_cc(self) -> Option<midi::CC> {
        match self {
            Parameter::Amp(param) => param.to_cc(),
            Parameter::Cabinet(param) => param.to_cc(),
            Parameter::Compressor(param) => param.to_cc(),
            Parameter::Effect(param) => param.to_cc(),
            Parameter::NoiseGate(param) => param.to_cc(),
            Parameter::UtilitySettings(param) => param.to_cc(),
        }
    }
}

impl CCParameterSetter for Dsp {
    type Parameter = Parameter;

    fn set_cc(&mut self, cc: midi::CC) -> Result<Option<Parameter>, Error> {
        macro_rules! try_set_cc {
            ($field:expr, $cc:ident, $variant:ident) => {
                match $field.set_cc($cc) {
                    Err(err) if err.is_unknown_cc() => (),
                    Ok(Some(param)) => return Ok(Some(Parameter::$variant(param))),
                    Ok(None) => return Ok(None),
                    Err(other) => return Err(other),
                }
            };
        }

        try_set_cc!(self.amp, cc, Amp);
        try_set_cc!(self.cabinet, cc, Cabinet);
        try_set_cc!(self.compressor, cc, Compressor);
        try_set_cc!(self.effect, cc, Effect);
        try_set_cc!(self.noise_gate, cc, NoiseGate);
        try_set_cc!(self.utility_settings, cc, UtilitySettings);

        Err(Error::CCNumberUnknown(cc.nb.as_u8()))
    }
}

fn fmt_percent(param: impl DiscreteParameter, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(normal) = param.normal() {
        f.write_fmt(format_args!("{:.0}", 100.0 * normal.as_f32()))
    } else {
        f.write_str("n/a")
    }
}

fn fmt_bipolar_normal(param: impl DiscreteParameter, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(normal) = param.normal() {
        let bipolar = 2.0 * normal.as_f32() - 1.0;
        f.write_fmt(format_args!("{:0.2}", bipolar))
    } else {
        f.write_str("n/a")
    }
}
