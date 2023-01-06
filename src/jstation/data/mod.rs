#[macro_use]
pub mod parameter;
pub use parameter::{
    BaseParameter, BoolParameter, CCParameter, CCParameterSetter, ConstRangeParameter,
    DiscreteParameter, DiscreteRange, Normal, ParameterNumber, ParameterSetter, RawParameterSetter,
    RawValue, VariableRange, VariableRangeParameter,
};

pub mod dsp;

pub mod program;
pub use program::{Program, ProgramData, ProgramId, ProgramNb, ProgramsBank};
