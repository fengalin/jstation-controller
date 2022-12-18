#[macro_use]
pub mod parameter;
pub use parameter::{
    BoolParameter, BoolRawParameter, CCParameter, CCParameterSetter, ConstRangeParameter,
    DiscreteParameter, DiscreteRange, DiscreteRawParameter, DiscreteValue, Normal, ParameterNumber,
    ParameterSetter, RawParameter, RawValue, VariableRange, VariableRangeParameter,
};

pub mod dsp;
pub use dsp::Parameter;

pub mod program;
pub use program::{Program, ProgramBank, ProgramNumber};
