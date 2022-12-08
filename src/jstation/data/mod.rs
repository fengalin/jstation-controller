#[macro_use]
pub mod parameter;
pub use parameter::{
    BoolCCParameter, BoolParameter, BoolRawParameter, DiscreteCCParameter, DiscreteParameter,
    DiscreteRange, DiscreteRawParameter, DiscreteValue, Normal, ParameterNumber, RawParameter,
    RawValue, ValueStatus,
};

pub mod dsp;
pub use dsp::Parameter;

pub mod program;
pub use program::{Program, ProgramBank, ProgramNumber};
