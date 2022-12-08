use crate::{
    jstation::data::{DiscreteParameter, Normal, ParameterNumber, RawValue},
    midi::CCNumber,
};

use std::fmt;

discrete_parameter!(Modeling {
    const DEFAULT = Normal::MIN,
    const MAX_RAW = RawValue::new(24),
    const PARAMETER_NB = ParameterNumber::new(9),
    const CC_NB = CCNumber::new(34),
});

static MODELINGS: [(&str, &str); (Modeling::MAX_RAW.as_u8() + 1) as usize] = [
    ("J Crunch", "JM150 Millennium Crunch"),
    ("J Solo", "JM150 Millennium Solo"),
    ("J Clean", "JM150 Millennium Clean"),
    ("Boutique", "Matchless DC30"),
    ("Rectified", "MesaBoogie Dual Rectifier"),
    ("Brit Stack", "Marshall JCM900"),
    ("Brit Class A", "'63 Vox AC30 top boost"),
    ("BlackFace", "'65 Fender Twin Reverb"),
    ("Boat Back", "piezo acoustic guitar"),
    ("Flat Top", "dreadnaught acoustic guitar"),
    ("Hot Rod", "Mesa Boogie Mark II C"),
    ("Tweed", "'57 Fender Tweed Deluxe"),
    ("Blues", "dynamic blues tube combo"),
    ("Fuzz", "60's fuzz tone"),
    ("Modern", "SWR bass"),
    ("British", "Trace Elliot bass amp"),
    ("Rock", "Ampeg SVT bass amp"),
    ("Hiwatt (A1)", "Hiwatt Custom 50"),
    ("Brit Master Vol (A2)", "'78 Marshall Mstr Volume"),
    ("Brit 800 EL84 (A3)", "'81 Marshall JCM800 w/EL34s"),
    ("Band Master (A4)", "'72 Fender Bandmaster"),
    ("Bass Man (A5)", "'65 Fender Bassman"),
    ("Stella Bass (A6)", "SWR Interstellar Odrive"),
    ("'83 Concert (A7)", "'83 Fender Concert Head"),
    ("Direct (A8)", "Direct - no modelling"),
];

impl fmt::Display for Modeling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let idx = self.to_raw_value().as_u8() as usize;
        fmt::Display::fmt(&(MODELINGS[idx].0), f)
    }
}

discrete_parameter!(
    #[derive(Display)]
    Gain {
        const DEFAULT = Normal::MIN,
        const MAX_RAW = RawValue::new(90),
        const PARAMETER_NB = ParameterNumber::new(10),
        const CC_NB = CCNumber::new(35),
    }
);

discrete_parameter!(
    #[derive(Display)]
    Treble {
        const DEFAULT = Normal::HALF,
        const MAX_RAW = RawValue::new(90),
        const PARAMETER_NB = ParameterNumber::new(11),
        const CC_NB = CCNumber::new(39),
    }
);

discrete_parameter!(
    #[derive(Display)]
    Middle {
        const DEFAULT = Normal::HALF,
        const MAX_RAW = RawValue::new(90),
        const PARAMETER_NB = ParameterNumber::new(12),
        const CC_NB = CCNumber::new(38),
    }
);

discrete_parameter!(
    #[derive(Display)]
    Bass {
        const DEFAULT = Normal::HALF,
        const MAX_RAW = RawValue::new(90),
        const PARAMETER_NB = ParameterNumber::new(13),
        const CC_NB = CCNumber::new(37),
    }
);

discrete_parameter!(
    #[derive(Display)]
    Level {
        const DEFAULT = Normal::MIN,
        const MAX_RAW = RawValue::new(90),
        const PARAMETER_NB = ParameterNumber::new(14),
        const CC_NB = CCNumber::new(36),
    }
);

#[derive(Clone, Copy, Debug, Default)]
pub struct Amp {
    pub modeling: Modeling,
    pub gain: Gain,
    pub treble: Treble,
    pub middle: Middle,
    pub bass: Bass,
    pub level: Level,
}
