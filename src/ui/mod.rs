use iced::Length;

// FIXME impl styles
pub const AMP_CABINET_LABEL_WIDTH: Length = Length::Units(85);
pub const CHECKBOX_SIZE: u16 = 16;
pub const COMBO_TEXT_SIZE: u16 = 15;
pub const DSP_TITLE_AREA_WIDTH: Length = Length::Units(276);
pub const KNOB_SIZE: Length = Length::Units(35);
pub const LABEL_TEXT_SIZE: u16 = 18;
pub const LABEL_WIDTH: Length = Length::Units(55);
pub const RADIO_SIZE: u16 = 16;
pub const RADIO_SPACING: u16 = 5;
pub const VALUE_TEXT_SIZE: u16 = 14;

pub mod widget;
pub use widget::{knob, switch};

pub mod amp;
pub mod cabinet;
pub mod compressor;
pub mod effect;
pub mod noise_gate;
pub mod utility_settings;
pub mod wah_expr;

pub mod app;
pub use app::{App, APP_NAME};

pub mod jstation;
pub mod midi;
