use std::fmt;

use crate::jstation::data::DiscreteParameter;
use jstation_derive::ParameterSetter;

#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct Delay {
    #[boolean(param_nb = 26, cc_nb = 52)]
    pub switch: Switch,
    #[const_range(max = 3, param_nb = 27, cc_nb = 53, display_map = name)]
    pub typ: Type,
    #[const_range(max = 99, param_nb = 28, cc_nb = 54, display_cents)]
    pub level: Level,
    // 100 ms increments.
    #[const_range(max = 30, param_nb = 29, cc_nb = 55)]
    pub time_course: TimeCourse,
    // 1 ms increments.
    #[const_range(max = 99, param_nb = 30, cc_nb = 56)]
    pub time_fine: TimeFine,
    #[const_range(max = 99, param_nb = 31, cc_nb = 57, display_cents)]
    pub feedback: Feedback,
}

const TYPE_NAMES: [&str; 4] = ["Mono", "Analog", "Pong", "Analog Pong"];

impl fmt::Display for TimeCourse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.raw_value().unwrap().as_u8();
        if value > 9 {
            f.write_fmt(format_args!("{:0.1} s", value as f32 / 10.0))
        } else {
            fmt::Display::fmt(&(value as u16 * 100), f)?;
            f.write_str(" ms")
        }
    }
}

impl fmt::Display for TimeFine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw_value().unwrap().as_u8(), f)?;
        f.write_str(" ms")
    }
}
