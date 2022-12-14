use std::fmt;

use crate::jstation::data::DiscreteParameter;
use jstation_derive::ParameterSetter;

#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct Compressor {
    #[boolean(param_nb = 0, cc_nb = 1)]
    pub switch: Switch,
    #[const_range(max = 50, param_nb = 1, cc_nb = 2)]
    pub threshold: Threshold,
    #[const_range(max = 9, param_nb = 2, cc_nb = 3, display_map = value)]
    pub ratio: Ratio,
    #[const_range(max = 30, param_nb = 3, cc_nb = 4)]
    pub gain: Gain,
    #[const_range(max = 19, param_nb = 4, cc_nb = 5, display_map = value)]
    pub freq: Freq,
}

impl fmt::Display for Threshold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let raw_value = self.raw_value().as_u8();

        // Don't display `-` if value is 0.
        let db_value = if raw_value > 0 {
            -1.0 * (raw_value as f32)
        } else {
            0.0
        };

        fmt::Display::fmt(&db_value, f)?;
        f.write_str(" dB")
    }
}

const RATIO_VALUES: [&str; 10] = [
    "1.1:1", "1.2:1", "1.5:1", "2:1", "3:1", "4:1", "6:1", "8:1", "10:1", "∞:1",
];

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.value(), f)
    }
}

impl fmt::Display for Gain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let db_value = self.raw_value().as_u8() as f32;

        fmt::Display::fmt(&db_value, f)?;
        f.write_str(" dB")
    }
}

const FREQ_VALUES: [&str; 20] = [
    "50 Hz", "63 Hz", "80 Hz", "100 Hz", "125 Hz", "160 Hz", "200 Hz", "250 Hz", "315 Hz",
    "400 Hz", "500 Hz", "630 Hz", "800 Hz", "1 kHz", "1.25 kHz", "1.6 kHz", "2 kHz", "2.5 kHz",
    "3.15 kHz", "full",
];

impl fmt::Display for Freq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.value(), f)
    }
}
