use std::{cmp, fmt};

use nom::IResult;

use crate::jstation::{
    data::{ParameterNumber, RawValue},
    take_split_bytes_u8, Error,
};
use crate::midi;

#[derive(Debug)]
pub struct Program {
    id: ProgramId,
    data: ProgramData,
}

impl Program {
    pub fn new(id: ProgramId, data: ProgramData) -> Self {
        Program { id, data }
    }

    pub fn id(&self) -> ProgramId {
        self.id
    }

    pub fn data(&self) -> &[RawValue] {
        self.data.data()
    }

    pub fn name(&self) -> &str {
        self.data.name()
    }
}

impl cmp::PartialEq<ProgramData> for Program {
    fn eq(&self, other: &ProgramData) -> bool {
        self.data.eq(other)
    }
}

#[derive(Clone, Debug)]
pub struct ProgramData {
    data: Box<[RawValue; Self::PARAM_COUNT]>,
    name: String,
}

impl ProgramData {
    pub const PARAM_COUNT: usize = (ParameterNumber::MAX.as_u8() + 1) as usize;
    const NAME_MAX_LEN: usize = 20;

    fn try_new(data: Box<[RawValue; Self::PARAM_COUNT]>, name: String) -> Result<Self, Error> {
        if name.len() > Self::NAME_MAX_LEN {
            return Err(Error::ProgramNameOutOfRange(name.len()));
        }

        Ok(ProgramData { data, name })
    }

    pub fn data(&self) -> &[RawValue; Self::PARAM_COUNT] {
        self.data.as_ref()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn format_name(&mut self, mut name: String) -> String {
        // Truncate the name so as to comply with the device's limits
        let buf = name.as_bytes();
        if buf.len() > Self::NAME_MAX_LEN {
            name = String::from_utf8_lossy(&buf[..Self::NAME_MAX_LEN]).to_string();
        }

        name
    }
}

impl cmp::PartialEq for ProgramData {
    fn eq(&self, other: &Self) -> bool {
        for (val1, val2) in self.data.iter().zip(other.data.iter()) {
            if val1 != val2 {
                return false;
            }
        }

        if self.name != other.name {
            return false;
        }

        true
    }
}

impl ProgramData {
    pub fn parse<'i>(
        input: &'i [u8],
        checksum: &mut u8,
        mut len: u16,
    ) -> IResult<&'i [u8], ProgramData> {
        let mut i = input;

        // FIXME could use new_uninit / assume_init if they were stable
        // since we will overide all zeros anyway.
        let mut data: Box<[RawValue; Self::PARAM_COUNT]> =
            [RawValue::ZERO; Self::PARAM_COUNT].into();

        for raw_value in data.iter_mut() {
            let (i_, byte) = take_split_bytes_u8(i, checksum)?;
            i = i_;
            *raw_value = byte.into();
        }

        len -= Self::PARAM_COUNT as u16;

        let mut name = vec![];
        let mut got_zero = false;

        for _ in 0..len {
            let (i_, byte) = take_split_bytes_u8(i, checksum)?;
            i = i_;
            if !got_zero {
                if byte != 0 {
                    name.push(byte);
                } else {
                    got_zero = true;
                }
            }
        }

        let name = String::from_utf8_lossy(&name).to_string();
        let prog_data = ProgramData::try_new(data, name).map_err(|err| {
            log::error!("ProgramData: {err}");

            nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::LengthValue,
            ))
        })?;

        Ok((i, prog_data))
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProgramNb(u8);

impl ProgramNb {
    const PRESET_BANKS: u8 = 10;
    const PRESETS: u8 = 3;

    pub fn preset_bank(self) -> u8 {
        self.0 / Self::PRESETS
    }

    pub fn preset(self) -> u8 {
        self.0 % Self::PRESETS + 1
    }

    pub fn enumerate() -> ProgramNbIter {
        ProgramNbIter { cur: 0 }
    }
}

impl TryFrom<u8> for ProgramNb {
    type Error = Error;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        if val >= Self::PRESET_BANKS * Self::PRESETS {
            return Err(Error::ProgramNumberOutOfRange(val));
        }

        Ok(ProgramNb(val))
    }
}

impl From<ProgramNb> for u8 {
    fn from(nb: ProgramNb) -> u8 {
        nb.0
    }
}

impl From<midi::ProgramNumber> for ProgramNb {
    fn from(midi_prog_nb: midi::ProgramNumber) -> Self {
        let midi_prog_nb: u8 = midi_prog_nb.into();

        ProgramNb(midi_prog_nb % (ProgramNb::PRESET_BANKS * ProgramNb::PRESETS))
    }
}

impl fmt::Display for ProgramNb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}.{}", self.preset_bank(), self.preset()))
    }
}

pub struct ProgramNbIter {
    cur: u8,
}

impl Iterator for ProgramNbIter {
    type Item = ProgramNb;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur;

        if cur == ProgramNb::PRESET_BANKS * ProgramNb::PRESETS {
            return None;
        }

        self.cur += 1;

        Some(ProgramNb(cur))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum ProgramsBank {
    #[default]
    User,
    Factory,
}

impl ProgramsBank {
    const USER: u8 = 1;
    const FACTORY: u8 = 0;

    pub fn is_user(self) -> bool {
        matches!(self, ProgramsBank::User)
    }

    pub fn into_prog_id(self, nb: ProgramNb) -> ProgramId {
        use ProgramsBank::*;
        match self {
            User => ProgramId::new_user(nb),
            Factory => ProgramId::new_factory(nb),
        }
    }

    fn midi_offset(self) -> u8 {
        use ProgramsBank::*;
        match self {
            User => 0,
            Factory => ProgramNb::PRESET_BANKS * ProgramNb::PRESETS,
        }
    }
}

impl TryFrom<u8> for ProgramsBank {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            Self::USER => Ok(ProgramsBank::User),
            Self::FACTORY => Ok(ProgramsBank::Factory),
            other => Err(Error::ProgramsBank(other)),
        }
    }
}

impl From<ProgramsBank> for u8 {
    fn from(progs_bank: ProgramsBank) -> Self {
        use ProgramsBank::*;
        match progs_bank {
            User => ProgramsBank::USER,
            Factory => ProgramsBank::FACTORY,
        }
    }
}

impl fmt::Display for ProgramsBank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ProgramsBank::*;
        f.write_str(match self {
            User => "User",
            Factory => "Factory",
        })
    }
}

impl From<midi::ProgramNumber> for ProgramsBank {
    fn from(midi_prog_nb: midi::ProgramNumber) -> Self {
        let midi_prog_nb: u8 = midi_prog_nb.into();

        if midi_prog_nb < ProgramNb::PRESET_BANKS * ProgramNb::PRESETS {
            ProgramsBank::User
        } else {
            ProgramsBank::Factory
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProgramId {
    progs_bank: ProgramsBank,
    nb: ProgramNb,
}

impl ProgramId {
    pub fn new(progs_bank: ProgramsBank, nb: ProgramNb) -> Self {
        ProgramId { progs_bank, nb }
    }

    pub fn new_user(nb: ProgramNb) -> Self {
        ProgramId {
            progs_bank: ProgramsBank::User,
            nb,
        }
    }

    pub fn new_factory(nb: ProgramNb) -> Self {
        ProgramId {
            progs_bank: ProgramsBank::Factory,
            nb,
        }
    }

    pub fn try_from_raw(progs_bank: u8, nb: u8) -> Result<Self, Error> {
        Ok(ProgramId {
            progs_bank: ProgramsBank::try_from(progs_bank)?,
            nb: ProgramNb::try_from(nb)?,
        })
    }

    pub fn progs_bank(self) -> ProgramsBank {
        self.progs_bank
    }

    pub fn nb(self) -> ProgramNb {
        self.nb
    }
}

impl From<midi::ProgramNumber> for ProgramId {
    fn from(midi_prog_nb: midi::ProgramNumber) -> Self {
        ProgramId {
            progs_bank: ProgramsBank::from(midi_prog_nb),
            nb: ProgramNb::from(midi_prog_nb),
        }
    }
}

impl From<ProgramId> for midi::ProgramNumber {
    fn from(id: ProgramId) -> Self {
        midi::ProgramNumber::from(id.nb.0 + id.progs_bank.midi_offset())
    }
}

impl fmt::Display for ProgramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.nb, f)?;
        f.write_fmt(format_args!("({})", self.progs_bank))
    }
}
