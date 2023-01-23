use jstation_derive::ParameterSetter;

#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct Expression {
    #[const_range(max = 14, param_nb = 40, cc_nb = 70, display_map = name)]
    pub assignment: Assignment,
    #[const_range(max = 127, param_nb = 41, cc_nb = 71, default_max, display_cents)]
    pub forward: Forward,
    #[const_range(max = 127, param_nb = 42, cc_nb = 72, display_cents)]
    pub back: Back,
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
