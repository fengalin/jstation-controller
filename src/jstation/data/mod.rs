#[macro_use]
pub mod parameter;
pub use parameter::{
    BoolParameter, CCParameter, CCParameterSetter, ConstRangeParameter, DiscreteParameter,
    DiscreteRange, Normal, ParameterNumber, ParameterSetter, RawValue, VariableRange,
    VariableRangeParameter,
};

pub mod dsp;

pub mod program;
pub use program::{Program, ProgramData, ProgramId, ProgramNb, ProgramParameter, ProgramsBank};
