use std::fmt;

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

    pub fn data(&self) -> &ProgramData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut ProgramData {
        &mut self.data
    }

    pub fn name(&self) -> &str {
        self.data.name()
    }
}

#[derive(Clone, Debug)]
pub struct ProgramData {
    buf: Box<[RawValue; Self::PARAM_COUNT]>,
    name: String,
}

impl ProgramData {
    // Safety: the link between `ParameterNumber::MAX` and `PARAM_COUNT`
    // is used as an invariant for optimizations in some operations.
    pub const PARAM_COUNT: usize = (ParameterNumber::MAX.as_u8() + 1) as usize;
    const NAME_MAX_LEN: usize = 40;

    fn try_new(buf: Box<[RawValue; Self::PARAM_COUNT]>, name: String) -> Result<Self, Error> {
        if name.as_bytes().len() > Self::NAME_MAX_LEN {
            return Err(Error::ProgramNameOutOfRange(name.len()));
        }

        Ok(ProgramData { buf, name })
    }

    #[inline]
    pub fn buf(&self) -> &[RawValue; Self::PARAM_COUNT] {
        self.buf.as_ref()
    }

    #[inline]
    pub fn buf_mut(&mut self) -> &mut [RawValue; Self::PARAM_COUNT] {
        self.buf.as_mut()
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn format_name(mut name: String) -> String {
        // Truncate the name so as to comply with the device's limits
        let buf = name.as_bytes();
        if buf.len() > Self::NAME_MAX_LEN {
            name = String::from_utf8_lossy(&buf[..Self::NAME_MAX_LEN]).to_string();
        }

        name
    }

    pub fn store_name(&mut self, name: &str) {
        self.name = Self::format_name(name.to_string());
    }

    pub fn serialize(&self) -> impl '_ + Iterator<Item = u8> {
        self.buf
            .iter()
            .map(u8::from)
            .chain(self.name.as_bytes().iter().cloned())
            // terminal \0 for name
            .chain(std::iter::once(0u8))
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

pub trait ProgramParameter {
    fn set_from(&mut self, data: &ProgramData) -> Result<(), Error>;

    /// Checks changes in `Self` compared to the provided `ProgramData`.
    ///
    /// Returns `true` if at least one Parameter has changed.
    fn has_changed(&self, data: &ProgramData) -> bool;

    /// Stores changes in `Self` to the provided `ProgramData`.
    ///
    /// `Self` may also change to comply with devices constraints
    /// such as the name length.
    fn store(&mut self, data: &mut ProgramData);
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

    pub fn is_factory(self) -> bool {
        matches!(self, ProgramsBank::Factory)
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
    fn from(bank: ProgramsBank) -> Self {
        use ProgramsBank::*;
        match bank {
            User => ProgramsBank::USER,
            Factory => ProgramsBank::FACTORY,
        }
    }
}

impl fmt::Display for ProgramsBank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ProgramsBank::*;
        f.write_str(match self {
            User => "User Bank",
            Factory => "Factory Bank",
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
    bank: ProgramsBank,
    nb: ProgramNb,
}

impl ProgramId {
    pub fn new(bank: ProgramsBank, nb: ProgramNb) -> Self {
        ProgramId { bank, nb }
    }

    pub fn new_user(nb: ProgramNb) -> Self {
        ProgramId {
            bank: ProgramsBank::User,
            nb,
        }
    }

    pub fn new_factory(nb: ProgramNb) -> Self {
        ProgramId {
            bank: ProgramsBank::Factory,
            nb,
        }
    }

    pub fn try_from_raw(bank: u8, nb: u8) -> Result<Self, Error> {
        Ok(ProgramId {
            bank: ProgramsBank::try_from(bank)?,
            nb: ProgramNb::try_from(nb)?,
        })
    }

    pub fn bank(self) -> ProgramsBank {
        self.bank
    }

    pub fn nb(self) -> ProgramNb {
        self.nb
    }
}

impl From<midi::ProgramNumber> for ProgramId {
    fn from(midi_prog_nb: midi::ProgramNumber) -> Self {
        ProgramId {
            bank: ProgramsBank::from(midi_prog_nb),
            nb: ProgramNb::from(midi_prog_nb),
        }
    }
}

impl From<ProgramId> for midi::ProgramNumber {
    fn from(id: ProgramId) -> Self {
        midi::ProgramNumber::from(id.nb.0 + id.bank.midi_offset())
    }
}

impl fmt::Display for ProgramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.nb, f)?;
        f.write_fmt(format_args!("({})", self.bank))
    }
}
