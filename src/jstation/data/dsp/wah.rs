use jstation_derive::ParameterSetter;

#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct Wah {
    #[boolean(param_nb = 5, cc_nb = 8)]
    pub switch: Switch,
    // Doc says "reserved for future use".
    //#[const_range(max = ?, param_nb = 6, cc_nb = 9)]
    //pub typ: Type,
    #[const_range(max = 127, param_nb = 7, cc_nb = 10, display_cents)]
    pub heel: Heel,
    #[const_range(max = 127, param_nb = 8, cc_nb = 11, default_max, display_cents)]
    pub toe: Toe,
}
