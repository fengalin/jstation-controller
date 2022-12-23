pub mod style;

pub mod widget;
pub use widget::{
    amp_cabinet_label, button, checkbox, dsp, dsp_keep_width, knob, modal, param_label, pick_list,
    radio, settings_checkbox, switch, toggler,
};

pub mod amp;
pub mod cabinet;
pub mod compressor;
pub mod delay;
pub mod effect;
pub mod noise_gate;
pub mod reverb;
pub mod utility_settings;
pub mod wah_expr;

pub mod app;
pub use app::{App, APP_NAME};

pub mod jstation;
pub mod midi;
