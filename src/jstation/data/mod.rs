#[macro_use]
pub mod parameter;
pub use parameter::{
    BoolParameter, CCParameter, CCParameterSetter, ConstRangeParameter, DiscreteParameter,
    DiscreteRange, Normal, ParameterNumber, ParameterSetter, RawParameter, RawValue, VariableRange,
    VariableRangeParameter,
};

pub mod dsp;
pub use dsp::Parameter;

pub mod program;
pub use program::{Program, ProgramBank, ProgramData, ProgramNumber};
