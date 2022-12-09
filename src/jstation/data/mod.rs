#[macro_use]
pub mod parameter;
pub use parameter::{
    BoolParameter, BoolRawParameter, CCParameter, DiscreteParameter, DiscreteRange,
    DiscreteRawParameter, DiscreteValue, Normal, ParameterNumber, ParameterSetter, RawParameter,
    RawValue,
};

pub mod dsp;
pub use dsp::Parameter;

pub mod program;
pub use program::{Program, ProgramBank, ProgramNumber};
