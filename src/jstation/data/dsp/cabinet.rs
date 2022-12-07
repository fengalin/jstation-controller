use crate::{
    jstation::data::{Normal, ParameterNumber, RawValue},
    midi::CCNumber,
};

discrete_parameter!(Cabinet {
    const DEFAULT = Normal::MIN,
    const MAX_RAW = RawValue::new(18),
    const PARAMETER_NB = ParameterNumber::new(15),
    const CC_NB = CCNumber::new(66),
});

pub static NAMES: [(&str, &str); 19] = [
    ("No Cabinet", ""),
    ("British 4x12", "Marshall 1960A with 75W Celestions"),
    ("Johnson 4x12", "loaded with Vintage 30W Celestions"),
    ("Fane 4x12", "Hiwatt SE4123 with Fanes",),
    ("Johnson 2x12", "Open back with Vintage 30W Celestions"),
    ("American 2x12", "Fender Twin 2x12"),
    ("Jennings Blue 2x12", "'63 Vox AC30"),
    ("Tweed 1x12", "Fender Deluxe 1x12"),
    ("Blonde 2x12", "Bassman 2x12"),
    ("Bass 4x10 with Tweeter", "SWR 4x10 with tweeter"),
    ("Bass 360 1x18", "Acoustic 360"),
    ("Flex Bass 1x15", "Ampeg Portaflex"),
    ("Green Back 4x12", "Marshall 1960B with 25W Celestion Greenbacks"),
    ("Mega 1516", "Peavy 1x15 and 2x8"),
    ("Boutique 4x12", "VHT 4x12 with Celestion V30s"),
    ("'65 Tweed 1x12", "'65 Fender Deluxe"),
    ("Goliath 4x10", "SWR Goliath"),
    ("Ivy League 1x10", "Fender Harvard"),
    ("Bass Man 4x10", "Fender Bassman"),
];
