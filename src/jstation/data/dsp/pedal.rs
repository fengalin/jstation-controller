use jstation_derive::ParameterSetter;

#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct Pedal {
    #[const_range(max = 127, cc_nb = 17)]
    pub expression: Expression,
    #[const_range(max = 127, cc_nb = 68)]
    pub volume: Volume,
}
