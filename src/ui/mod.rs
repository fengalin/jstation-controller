use iced::Length;

#[macro_use]
mod macros;

// FIXME impl styles
pub const AMP_CABINET_LABEL_WIDTH: Length = Length::Units(85);
pub const CHECKBOX_SIZE: u16 = 16;
pub const COMBO_TEXT_SIZE: u16 = 15;
pub const DSP_TITLE_AREA_WIDTH: Length = Length::Units(276);
pub const KNOB_SIZE: Length = Length::Units(35);
pub const LABEL_TEXT_SIZE: u16 = 18;
pub const LABEL_WIDTH: Length = Length::Units(55);
pub const VALUE_TEXT_SIZE: u16 = 14;

pub mod amp;
pub mod cabinet;
pub mod compressor;
pub mod noise_gate;

pub mod app;
pub use app::{App, APP_NAME};

pub mod jstation;
pub mod midi;
pub mod utility_settings;

fn to_ui_normal(normal: crate::jstation::data::Normal) -> iced_audio::Normal {
    // Safety: these two `Normal`s are both wrappers on `f32`
    // and they observe the same invariants: the inner `f32` is constrained
    // to (0.0..=1.0).
    unsafe { std::mem::transmute(normal) }
}

fn to_ui_param<P>(param: P) -> iced_audio::NormalParam
where
    P: crate::jstation::data::DiscreteParameter,
{
    let value = to_ui_normal(param.normal());
    let default = to_ui_normal(P::DEFAULT);

    iced_audio::NormalParam { value, default }
}

fn to_jstation_normal(normal: iced_audio::Normal) -> crate::jstation::data::Normal {
    // Safety: these two `Normal`s are both wrappers on `f32`
    // and they observe the same invariants: the inner `f32` is constrained
    // to (0.0..=1.0).
    unsafe { std::mem::transmute(normal) }
}

#[cfg(test)]
mod tests {
    #[test]
    fn to_ui_normal() {
        use super::to_ui_normal;
        use crate::jstation::data::Normal;

        const JS_MIN: Normal = Normal::MIN;
        const JS_CENTER: Normal = Normal::CENTER;
        const JS_MAX: Normal = Normal::MAX;

        assert_eq!(to_ui_normal(JS_MIN).as_f32(), JS_MIN.as_f32());
        assert_eq!(to_ui_normal(JS_CENTER).as_f32(), JS_CENTER.as_f32());
        assert_eq!(to_ui_normal(JS_MAX).as_f32(), JS_MAX.as_f32());

        let less_than_min_res = Normal::try_from(0.0 - f32::EPSILON);
        assert!(less_than_min_res.is_err());

        let more_than_max_res = Normal::try_from(1.0 + f32::EPSILON);
        assert!(more_than_max_res.is_err());
    }

    #[test]
    fn to_jstation_normal() {
        use super::to_jstation_normal;
        use iced_audio::Normal;

        const UI_MIN: Normal = Normal::MIN;
        const UI_CENTER: Normal = Normal::CENTER;
        const UI_MAX: Normal = Normal::MAX;

        assert_eq!(to_jstation_normal(UI_MIN).as_f32(), UI_MIN.as_f32());
        assert_eq!(to_jstation_normal(UI_CENTER).as_f32(), UI_CENTER.as_f32());
        assert_eq!(to_jstation_normal(UI_MAX).as_f32(), UI_MAX.as_f32());

        let clipped_less_than_min = Normal::from_clipped(0.0 - f32::EPSILON);
        assert_eq!(
            to_jstation_normal(clipped_less_than_min).as_f32(),
            UI_MIN.as_f32()
        );

        let clipped_more_than_max = Normal::from_clipped(1.0 + f32::EPSILON);
        assert_eq!(
            to_jstation_normal(clipped_more_than_max).as_f32(),
            UI_MAX.as_f32()
        );
    }
}
