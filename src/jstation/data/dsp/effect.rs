use std::fmt;

use jstation_derive::ParameterSetter;

use crate::jstation::data::{
    DiscreteParameter, DiscreteRange, Normal, RawValue, VariableRange, VariableRangeParameter,
};

#[derive(Clone, Copy, Debug, Default, ParameterSetter)]
pub struct Effect {
    #[boolean(param_nb = 19, cc_nb = 44)]
    pub switch: Switch,
    #[const_range(discriminant, max = 6, param_nb = 20, cc_nb = 45, display_map = name)]
    pub typ: Type,
    #[const_range(max = 99, param_nb = 21, cc_nb = 46, display_cents)]
    pub mix: Mix,
    // The speed parameter changes assignment depending on the effect type:
    // - For Auto Wah, it's the WahType with 3 possible values.
    // - For Pitch/Detune, it's the Pitch: max = 48, bipolar -24 to +24 semitones.
    // - For the other effects, it's the Speed : max = 99, no specific unit.
    #[variable_range(param_nb = 22, cc_nb = 47)]
    pub speed: Speed,
    // The depth parameter changes assignment depending on the effect type:
    // - For Pitch/Detune, it's the Detune: max = 60, bipolar -30 to +30 cents.
    // - For the other effects, it's the Depth : max = 99, no specific unit.
    #[variable_range(param_nb = 23, cc_nb = 48)]
    pub depth: Depth,
    // The regen parameter changes assignment depending on the effect type:
    // - For Chorus: max is 40, no specific unit.
    // - No Regen for Tremolo, Rotary Speaker, Auto Wah and Pitch/Detune.
    // - For the other effects: max = 99, no specific unit.
    #[variable_range(param_nb = 24, cc_nb = 49)]
    pub regen: Regen,
    #[boolean(param_nb = 25, cc_nb = 50)]
    pub post: Post,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Discriminant {
    #[default]
    Chorus,
    Flanger,
    Phaser,
    Tremolo,
    RotarySpeaker,
    AutoWah,
    PitchDetune,
}

impl From<Type> for Discriminant {
    fn from(typ: Type) -> Self {
        use Discriminant::*;
        match typ.raw_value().unwrap().as_u8() {
            0 => Chorus,
            1 => Flanger,
            2 => Phaser,
            3 => Tremolo,
            4 => RotarySpeaker,
            5 => AutoWah,
            6 => PitchDetune,
            _ => panic!("Effect Type / Discriminant mismatch"),
        }
    }
}

const TYPE_NAMES: [&str; 7] = [
    "Chorus",
    "Flanger",
    "Phaser",
    "Tremolo",
    "Rotary Speaker",
    "Auto Wah",
    "Pitch / Detune",
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SpeedAssignment {
    #[default]
    Speed,
    WahType,
    Semitones,
}

impl SpeedAssignment {
    pub const fn name(self) -> &'static str {
        use SpeedAssignment::*;
        match self {
            Speed => "Speed",
            WahType => "Wah Type",
            Semitones => "Semitones",
        }
    }
}

impl fmt::Display for SpeedAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl VariableRange for Speed {
    type Discriminant = Discriminant;

    fn range_from(discr: Self::Discriminant) -> Option<DiscreteRange> {
        use Discriminant::*;
        Some(match discr {
            AutoWah => DiscreteRange::new(RawValue::new(0), RawValue::new(2)),
            PitchDetune => DiscreteRange::new(RawValue::new(0), RawValue::new(48)),
            _ => DiscreteRange::new(RawValue::new(0), RawValue::new(99)),
        })
    }
}

impl DiscreteParameter for Speed {
    fn name(self) -> &'static str {
        self.assignment().name()
    }

    fn normal_default(self) -> Option<Normal> {
        Some(match self.discr {
            Discriminant::PitchDetune => Normal::CENTER,
            _ => Normal::MIN,
        })
    }

    fn normal(self) -> Option<Normal> {
        let range = self.range().unwrap();
        Some(range.try_normalize(self.value).unwrap())
    }

    fn raw_value(self) -> Option<RawValue> {
        Some(self.value)
    }

    fn reset(&mut self) -> Option<Self> {
        let default = RawValue::new(match self.discr {
            Discriminant::PitchDetune => 24,
            _ => 0,
        });

        if self.value == default {
            return None;
        }

        self.value = default;

        Some(*self)
    }
}

impl Speed {
    /// Returns the assignment for the `Speed` parameter according to current `Type`.
    pub fn assignment(self) -> SpeedAssignment {
        match self.discr {
            Discriminant::AutoWah => SpeedAssignment::WahType,
            Discriminant::PitchDetune => SpeedAssignment::Semitones,
            _ => SpeedAssignment::Speed,
        }
    }
}

impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self.discr {
            Discriminant::AutoWah => self.raw_value().unwrap().as_u8() as i32,
            Discriminant::PitchDetune => {
                // -24 to +24 semitones.
                self.raw_value().unwrap().as_u8() as i32 - 24i32
            }
            _ => self.range().unwrap().to_cents(self.value).unwrap() as i32,
        };

        fmt::Display::fmt(&value, f)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DepthAssignment {
    #[default]
    Depth,
    Detune,
}

impl DepthAssignment {
    pub const fn name(self) -> &'static str {
        use DepthAssignment::*;
        match self {
            Depth => "Depth",
            Detune => "Detune",
        }
    }
}

impl fmt::Display for DepthAssignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Depth {
    /// Returns the assignment for the `Depth` parameter according to current `Type`.
    pub fn assignment(self) -> DepthAssignment {
        match self.discr {
            Discriminant::PitchDetune => DepthAssignment::Detune,
            _ => DepthAssignment::Depth,
        }
    }
}

impl VariableRange for Depth {
    type Discriminant = Discriminant;

    fn range_from(discr: Self::Discriminant) -> Option<DiscreteRange> {
        use Discriminant::*;
        Some(match discr {
            PitchDetune => DiscreteRange::new(RawValue::new(0), RawValue::new(60)),
            _ => DiscreteRange::new(RawValue::new(0), RawValue::new(99)),
        })
    }
}

impl DiscreteParameter for Depth {
    fn name(self) -> &'static str {
        self.assignment().name()
    }

    fn normal_default(self) -> Option<Normal> {
        Some(match self.discr {
            Discriminant::PitchDetune => Normal::CENTER,
            _ => Normal::MIN,
        })
    }

    fn normal(self) -> Option<Normal> {
        let range = self.range().unwrap();
        Some(range.try_normalize(self.value).unwrap())
    }

    fn raw_value(self) -> Option<RawValue> {
        Some(self.value)
    }

    fn reset(&mut self) -> Option<Self> {
        let default = RawValue::new(match self.discr {
            Discriminant::PitchDetune => 30,
            _ => 0,
        });

        if self.value == default {
            return None;
        }

        self.value = default;

        Some(*self)
    }
}

impl fmt::Display for Depth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self.discr {
            Discriminant::PitchDetune => {
                // -30 to +30 cents.
                self.raw_value().unwrap().as_u8() as i32 - 30i32
            }
            _ => self.range().unwrap().to_cents(self.value).unwrap() as i32,
        };

        fmt::Display::fmt(&value, f)
    }
}

impl VariableRange for Regen {
    type Discriminant = Discriminant;

    fn range_from(discr: Self::Discriminant) -> Option<DiscreteRange> {
        use Discriminant::*;
        match discr {
            Chorus => Some(DiscreteRange::new(RawValue::new(0), RawValue::new(40))),
            Flanger | Phaser => Some(DiscreteRange::new(RawValue::new(0), RawValue::new(99))),
            _ => None,
        }
    }
}

impl DiscreteParameter for Regen {
    fn name(self) -> &'static str {
        use Discriminant::*;
        match self.discr {
            Chorus | Flanger | Phaser => "Regen",
            _ => "N/A",
        }
    }

    fn normal_default(self) -> Option<Normal> {
        if self.is_active() {
            Some(Normal::MIN)
        } else {
            None
        }
    }

    fn normal(self) -> Option<Normal> {
        let range = self.range()?;
        Some(range.try_normalize(self.value).unwrap())
    }

    fn raw_value(self) -> Option<RawValue> {
        if !self.is_active() {
            return None;
        }

        Some(self.value)
    }

    fn reset(&mut self) -> Option<Self> {
        if !self.is_active() {
            return None;
        }

        if self.value == RawValue::ZERO {
            return None;
        }

        self.value = RawValue::ZERO;

        Some(*self)
    }

    fn is_active(self) -> bool {
        use Discriminant::*;
        matches!(self.discr, Chorus | Flanger | Phaser)
    }
}

impl fmt::Display for Regen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(range) = self.range() {
            fmt::Display::fmt(&range.to_cents(self.value).unwrap(), f)
        } else {
            f.write_str("n/a")
        }
    }
}
