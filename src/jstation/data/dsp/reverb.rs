use std::fmt;

use jstation_derive::ParameterSetter;

#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct Reverb {
    #[boolean(param_nb = 32, cc_nb = 59)]
    pub switch: Switch,
    #[const_range(max = 12, param_nb = 33, cc_nb = 60, display_map = name)]
    pub typ: Type,
    #[const_range(max = 99, param_nb = 34, cc_nb = 61, display_cents)]
    pub level: Level,
    #[const_range(max = 99, param_nb = 35, cc_nb = 62, display_cents)]
    pub diffusion: Diffusion,
    #[const_range(max = 99, param_nb = 36, cc_nb = 63, display_cents)]
    pub density: Density,
    #[const_range(max = 9, param_nb = 37, cc_nb = 65, display_raw)]
    pub decay: Decay,
}

const TYPE_NAMES: [&str; 13] = [
    "Club",
    "Studio",
    "Bathroom",
    "Plate",
    "Sound Stage",
    "Garage",
    "Hall",
    "Church",
    "Arena",
    "2x 7\" springs",
    "2x 14\" springs",
    "3x 14\" springs",
    "Rattle & Boing",
];
