use std::fmt;

use crate::jstation::data::DiscreteParameter;
use jstation_derive::ParameterSetter;

#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct Amp {
    #[const_range(max = 24, param_nb = 9, cc_nb = 34, display_map = name, display_map = nick)]
    pub modeling: Modeling,
    #[const_range(max = 90, param_nb = 10, cc_nb = 35, display_cents)]
    pub gain: Gain,
    #[const_range(max = 90, param_nb = 11, cc_nb = 39, default_center)]
    pub treble: Treble,
    #[const_range(max = 90, param_nb = 12, cc_nb = 38, default_center)]
    pub middle: Middle,
    #[const_range(max = 90, param_nb = 13, cc_nb = 37, default_center)]
    pub bass: Bass,
    #[const_range(max = 90, param_nb = 14, cc_nb = 36, display_cents)]
    pub level: Level,
}

const MODELING_NAMES: [&str; 25] = [
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
];

const MODELING_NICKS: [&str; 25] = [
    "J Crunch",
    "J Solo",
    "J Clean",
    "Boutique",
    "Rectified",
    "Brit Stack",
    "Brit Combo",
    "Black Face",
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
];

fn fmt_bipolar_normal(param: impl DiscreteParameter, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if let Some(normal) = param.normal() {
        let bipolar = 20.0 * normal.as_ratio() - 10.0;
        f.write_fmt(format_args!("{:0.1}", bipolar))
    } else {
        f.write_str("n/a")
    }
}

impl fmt::Display for Bass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_bipolar_normal(*self, f)
    }
}

impl fmt::Display for Middle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_bipolar_normal(*self, f)
    }
}

impl fmt::Display for Treble {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_bipolar_normal(*self, f)
    }
}
