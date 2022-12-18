use std::fmt;

use jstation_derive::ParameterSetter;

#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct WahExpr {
    #[boolean(param_nb = 5, cc_nb = 8)]
    pub switch: Switch,
    // Doc says "reserved for future use".
    //#[const_range(max = ?, param_nb = 6, cc_nb = 9)]
    //pub typ: Type,
    #[const_range(max = 127, param_nb = 7, cc_nb = 10)]
    pub heel: Heel,
    #[const_range(max = 127, param_nb = 8, cc_nb = 11)]
    pub toe: Toe,
    #[const_range(max = 14, param_nb = 40, cc_nb = 70, display_map = name)]
    pub assignment: Assignment,
    #[const_range(max = 127, param_nb = 41, cc_nb = 71)]
    pub forward: Forward,
    #[const_range(max = 127, param_nb = 42, cc_nb = 72)]
    pub back: Back,
}

impl fmt::Display for Heel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        super::fmt_percent(*self, f)
    }
}

impl fmt::Display for Toe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        super::fmt_percent(*self, f)
    }
}

const ASSIGNMENT_NAMES: [&str; 15] = [
    "No Link",
    "Master Level",
    "Volume Pedal",
    "Gain",
    "Treble",
    "Middle",
    "Bass",
    "Amp Level",
    "Effect Level",
    "Effect Speed / Pitch",
    "Effect Depth / Detune",
    "Effect Regen",
    "Delay Level",
    "Delay Feedback",
    "Reverb Level",
];

impl fmt::Display for Forward {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        super::fmt_percent(*self, f)
    }
}

impl fmt::Display for Back {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        super::fmt_percent(*self, f)
    }
}
