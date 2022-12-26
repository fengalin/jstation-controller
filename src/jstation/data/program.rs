use std::{borrow::Cow, cmp, fmt};

use nom::IResult;

use crate::jstation::{
    data::{ParameterNumber, RawValue},
    take_split_bytes_u8, Error,
};
use crate::midi;

#[derive(Debug)]
pub struct Program {
    bank: ProgramBank,
    nb: ProgramNumber,

    original: ProgramData,
    cur: ProgramData,
}

impl Program {
    pub fn new(bank: ProgramBank, nb: ProgramNumber, data: ProgramData) -> Self {
        let original = data;

        Program {
            bank,
            nb,
            cur: original.clone(),
            original,
        }
    }

    pub fn bank(&self) -> ProgramBank {
        self.bank
    }

    pub fn nb(&self) -> ProgramNumber {
        self.nb
    }

    pub fn data(&self) -> &[RawValue] {
        &self.cur.data
    }

    pub fn name(&self) -> &str {
        self.cur.name.as_ref()
    }

    pub fn has_changed(&self) -> bool {
        self.cur.has_changed()
    }

    pub fn undo(&mut self) {
        self.cur = self.original.clone();
    }

    pub fn apply(&mut self) {
        if matches!(self.cur.data, Cow::Owned(_)) {
            std::mem::swap(&mut self.original.data, &mut self.cur.data);
            self.cur.data = self.original.data.clone();
        }

        assert!(matches!(self.cur.data, Cow::Borrowed(_)));

        if matches!(self.cur.name, Cow::Owned(_)) {
            std::mem::swap(&mut self.original.name, &mut self.cur.name);
            self.cur.name = self.original.name.clone();
        }

        assert!(matches!(self.cur.name, Cow::Borrowed(_)));
    }

    pub fn rename_as(&mut self, name: String) {
        self.cur.rename_as(&self.original.name, name);
    }
}

impl cmp::PartialEq<ProgramData> for Program {
    fn eq(&self, other: &ProgramData) -> bool {
        self.original.eq(other)
    }
}

#[derive(Clone, Debug)]
pub struct ProgramData {
    data: Cow<'static, [RawValue]>,
    name: Cow<'static, str>,
}

impl ProgramData {
    pub const PARAM_COUNT: usize = (ParameterNumber::MAX.as_u8() + 1) as usize;
    const NAME_MAX_LEN: usize = 20;

    pub fn try_new(data: Vec<RawValue>, name: String) -> Result<Self, Error> {
        if data.len() > Self::PARAM_COUNT {
            return Err(Error::ProgramDataOutOfRange(data.len()));
        }

        if name.len() > Self::NAME_MAX_LEN {
            return Err(Error::ProgramNameOutOfRange(name.len()));
        }

        let data = Cow::<[RawValue]>::from(data);
        let name = Cow::<str>::from(name);

        Ok(ProgramData { data, name })
    }

    pub fn data(&self) -> &[RawValue] {
        &self.data
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    // FIXME need to check whether a changed param value is back to
    // its original value to figure out it has changed.
    fn has_changed(&self) -> bool {
        matches!(self.data, Cow::Owned(_)) | matches!(self.name, Cow::Owned(_))
    }

    #[allow(clippy::ptr_arg)]
    fn rename_as(&mut self, original_name: &Cow<'static, str>, mut name: String) {
        // Truncate the name so as to comply with the device's limits
        let buf = name.as_bytes();
        if buf.len() > Self::NAME_MAX_LEN {
            name = String::from_utf8_lossy(&buf[..Self::NAME_MAX_LEN]).to_string();
        }

        if name == self.name {
            return;
        }

        if name == *original_name {
            if matches!(self.name, Cow::Owned(_)) {
                // renaming as the original name
                self.name = original_name.clone();
            }

            return;
        }

        self.name = Cow::<str>::from(name);
    }
}

impl cmp::PartialEq for ProgramData {
    fn eq(&self, other: &Self) -> bool {
        if self.name.as_ref() != other.name.as_ref() {
            return false;
        }

        for data in self.data.iter().zip(other.data.iter()) {
            if data.0 != data.1 {
                return false;
            }
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

        let mut data = Vec::<RawValue>::new();
        for _ in 0..Self::PARAM_COUNT {
            let (i_, byte) = take_split_bytes_u8(i, checksum)?;
            i = i_;
            data.push(byte.into());
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProgramBank {
    Factory,
    User,
    Unknown(u8),
}

impl fmt::Display for ProgramBank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgramBank::User => f.write_str("user"),
            ProgramBank::Factory => f.write_str("factory"),
            ProgramBank::Unknown(other) => write!(f, "unknown ({other})"),
        }
    }
}

impl From<u8> for ProgramBank {
    fn from(val: u8) -> Self {
        match val {
            1 => ProgramBank::User,
            0 => ProgramBank::Factory,
            other => ProgramBank::Unknown(other),
        }
    }
}

impl From<ProgramBank> for u8 {
    fn from(val: ProgramBank) -> Self {
        match val {
            ProgramBank::User => 1,
            ProgramBank::Factory => 0,
            ProgramBank::Unknown(other) => other,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct ProgramNumber(u8);

impl ProgramNumber {
    pub fn as_u8(self) -> u8 {
        self.0
    }
}

impl From<u8> for ProgramNumber {
    fn from(nb: u8) -> Self {
        ProgramNumber(nb)
    }
}

impl From<ProgramNumber> for u8 {
    fn from(nb: ProgramNumber) -> Self {
        nb.0
    }
}

impl From<midi::ProgramNumber> for ProgramNumber {
    fn from(nb: midi::ProgramNumber) -> Self {
        // FIXME need to return ProgramBank when nb > 29
        ProgramNumber(u8::from(nb))
    }
}

impl From<ProgramNumber> for midi::ProgramNumber {
    fn from(nb: ProgramNumber) -> Self {
        // FIXME need to also use ProgramBank
        nb.0.into()
    }
}

impl fmt::Display for ProgramNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}.{}", self.0 / 3, self.0 % 3 + 1))
    }
}
