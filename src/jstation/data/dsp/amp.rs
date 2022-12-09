use crate::{
    jstation::data::{Normal, ParameterNumber, RawValue},
    midi::CCNumber,
};

discrete_parameter!(Modeling {
    const DEFAULT = Normal::MIN,
    const MAX_RAW = RawValue::new(24),
    const PARAMETER_NB = ParameterNumber::new(9),
    const CC_NB = CCNumber::new(34),
});

generate_parameter_list!(
    Modeling,
    ModelingNick,
    nick,
    nicks,
    [
        "J Crunch",
        "J Solo",
        "J Clean",
        "Boutique",
        "Rectified",
        "Brit Stack",
        "Brit Class A",
        "BlackFace",
        "Boat Back",
        "Flat Top",
        "Hot Rod",
        "Tweed",
        "Blues",
        "Fuzz",
        "Modern",
        "British",
        "Rock",
        "Hiwatt (A1)",
        "Brit Master Vol (A2)",
        "Brit 800 EL84 (A3)",
        "Band Master (A4)",
        "Bass Man (A5)",
        "Stella Bass (A6)",
        "'83 Concert (A7)",
        "Direct (A8)",
    ],
);

generate_parameter_list!(
    Modeling,
    ModelingName,
    name,
    names,
    [
        "JM150 Millennium Crunch",
        "JM150 Millennium Solo",
        "JM150 Millennium Clean",
        "Matchless DC30",
        "MesaBoogie Dual Rectifier",
        "Marshall JCM900",
        "'63 Vox AC30 top boost",
        "'65 Fender Twin Reverb",
        "piezo acoustic guitar",
        "dreadnaught acoustic guitar",
        "Mesa Boogie Mark II C",
        "'57 Fender Tweed Deluxe",
        "dynamic blues tube combo",
        "60's fuzz tone",
        "SWR bass",
        "Trace Elliot bass amp",
        "Ampeg SVT bass amp",
        "Hiwatt Custom 50",
        "'78 Marshall Mstr Volume",
        "'81 Marshall JCM800 with EL34s",
        "'72 Fender Bandmaster",
        "'65 Fender Bassman",
        "SWR Interstellar Odrive",
        "'83 Fender Concert Head",
        "Direct - no modelling",
    ],
);

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
