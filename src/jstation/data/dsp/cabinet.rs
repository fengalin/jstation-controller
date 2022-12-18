use jstation_derive::ParameterGroup;

#[derive(Clone, Copy, Debug, Default, ParameterGroup)]
pub struct Cabinet {
    #[const_range(max = 18, param_nb = 15, cc_nb = 66, display_map = name, display_map = nick)]
    pub typ: Type,
}

const TYPE_NAMES: [&str; 19] = [
    "No Cabinet",
    "Marshall 1960A with 75W Celestions",
    "loaded with Vintage 30W Celestions",
    "Hiwatt SE4123 with Fanes",
    "Open back with Vintage 30W Celestions",
    "Fender Twin 2x12",
    "'63 Vox AC30",
    "Fender Deluxe 1x12",
    "Bassman 2x12",
    "SWR 4x10 with tweeter",
    "Acoustic 360",
    "Ampeg Portaflex",
    "Marshall 1960B with 25W Celestion Greenbacks",
    "Peavy 1x15 and 2x8",
    "VHT 4x12 with Celestion V30s",
    "'65 Fender Deluxe",
    "SWR Goliath",
    "Fender Harvard",
    "Fender Bassman",
];

const TYPE_NICKS: [&str; 19] = [
    "No Cabinet",
    "British 4x12",
    "Johnson 4x12",
    "Fane 4x12",
    "Johnson 2x12",
    "American 2x12",
    "Jennings Blue 2x12",
    "Tweed 1x12",
    "Blonde 2x12",
    "Bass 4x10 with Tweeter",
    "Bass 360 1x18",
    "Flex Bass 1x15",
    "Green Back 4x12",
    "Mega 1516",
    "Boutique 4x12",
    "'65 Tweed 1x12",
    "Goliath 4x10",
    "Ivy League 1x10",
    "Bass Man 4x10",
];
