use jstation_derive::ParamGroup;

#[derive(Debug, Default, ParamGroup)]
pub struct Compressor {
    #[boolean(param_nb = 0, cc_nb = 1, display_raw)]
    pub switch: Switch,
    #[discrete(max = 50, param_nb = 1, cc_nb = 2, display_raw)]
    pub threshold: Threshold,
    #[discrete(max = 9, param_nb = 2, cc_nb = 3, display_map = value)]
    pub ratio: Ratio,
    #[discrete(max = 30, param_nb = 3, cc_nb = 4, display_raw)]
    pub gain: Gain,
    #[discrete(max = 19, param_nb = 4, cc_nb = 5, name = "Max. Freq.", display_map = value)]
    pub freq: Freq,
}

const RATIO_VALUES: [&str; 10] = [
    "1.1:1",
    "1.2:1",
    "1.5:1",
    "2:1",
    "3:1",
    "4:1",
    "6:1",
    "8:1",
    "10:1",
    "infinity:1",
];

const FREQ_VALUES: [&str; 20] = [
    "50 Hz",
    "63 Hz",
    "80 Hz",
    "100 Hz",
    "125 Hz",
    "160 Hz",
    "200 Hz",
    "250 Hz",
    "315 Hz",
    "400 Hz",
    "500 Hz",
    "630 Hz",
    "800 Hz",
    "1 kHz",
    "1.25 kHz",
    "1.6 kHz",
    "2 kHz",
    "2.5 kHz",
    "3.15 kHz",
    "Full Range",
];
