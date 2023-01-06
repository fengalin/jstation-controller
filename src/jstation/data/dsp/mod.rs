use crate::{
    jstation::{
        data::{CCParameter, CCParameterSetter, ParameterSetter, ProgramData, ProgramParameter},
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

pub mod delay;
pub use delay::Delay;

pub mod effect;
pub use effect::Effect;

pub mod noise_gate;
pub use noise_gate::NoiseGate;

pub mod reverb;
pub use reverb::Reverb;

pub mod utility_settings;
pub use utility_settings::UtilitySettings;

pub mod wah_expr;
pub use wah_expr::WahExpr;

#[derive(Debug, Default)]
pub struct Dsp {
    pub name: String,
    pub amp: Amp,
    pub cabinet: Cabinet,
    pub compressor: Compressor,
    pub delay: Delay,
    pub effect: Effect,
    pub noise_gate: NoiseGate,
    pub reverb: Reverb,
    pub wah_expr: WahExpr,
    pub utility_settings: UtilitySettings,
}

impl ProgramParameter for Dsp {
    fn set_from(&mut self, data: &ProgramData) -> Result<(), Error> {
        self.name = data.name().to_string();
        self.compressor.set_from(data)?;
        self.wah_expr.set_from(data)?;
        self.amp.set_from(data)?;
        self.cabinet.set_from(data)?;
        self.noise_gate.set_from(data)?;
        self.effect.set_from(data)?;
        self.delay.set_from(data)?;
        self.reverb.set_from(data)
    }

    fn has_changed(&self, original: &ProgramData) -> bool {
        self.compressor.has_changed(original)
            || self.wah_expr.has_changed(original)
            || self.amp.has_changed(original)
            || self.cabinet.has_changed(original)
            || self.noise_gate.has_changed(original)
            || self.effect.has_changed(original)
            || self.delay.has_changed(original)
            || self.reverb.has_changed(original)
            || self.name.as_str() != original.name()
    }

    fn store(&mut self, data: &mut ProgramData) {
        self.compressor.store(data);
        self.wah_expr.store(data);
        self.amp.store(data);
        self.cabinet.store(data);
        self.noise_gate.store(data);
        self.effect.store(data);
        self.delay.store(data);
        self.reverb.store(data);

        // FIXME find a solution to reduce String clones
        data.store_name(&self.name);
        self.name = data.name().to_string();
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Parameter {
    Amp(amp::Parameter),
    Cabinet(cabinet::Parameter),
    Compressor(compressor::Parameter),
    Delay(delay::Parameter),
    Effect(effect::Parameter),
    NoiseGate(noise_gate::Parameter),
    Reverb(reverb::Parameter),
    UtilitySettings(utility_settings::Parameter),
    WahExpr(wah_expr::Parameter),
}

impl ParameterSetter for Dsp {
    type Parameter = Parameter;

    fn set(&mut self, new: Parameter) -> Option<Parameter> {
        use Parameter::*;
        match new {
            Amp(param) => self.amp.set(param).map(Parameter::from),
            Cabinet(param) => self.cabinet.set(param).map(Parameter::from),
            Compressor(param) => self.compressor.set(param).map(Parameter::from),
            Delay(param) => self.delay.set(param).map(Parameter::from),
            Effect(param) => self.effect.set(param).map(Parameter::from),
            NoiseGate(param) => self.noise_gate.set(param).map(Parameter::from),
            Reverb(param) => self.reverb.set(param).map(Parameter::from),
            WahExpr(param) => self.wah_expr.set(param).map(Parameter::from),
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
            Parameter::Delay(param) => param.to_cc(),
            Parameter::Effect(param) => param.to_cc(),
            Parameter::NoiseGate(param) => param.to_cc(),
            Parameter::Reverb(param) => param.to_cc(),
            Parameter::WahExpr(param) => param.to_cc(),
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
        try_set_cc!(self.delay, cc, Delay);
        try_set_cc!(self.effect, cc, Effect);
        try_set_cc!(self.noise_gate, cc, NoiseGate);
        try_set_cc!(self.reverb, cc, Reverb);
        try_set_cc!(self.wah_expr, cc, WahExpr);
        try_set_cc!(self.utility_settings, cc, UtilitySettings);

        Err(Error::CCNumberUnknown(cc.nb.as_u8()))
    }
}
