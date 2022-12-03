use std::{borrow::Cow, fmt};

pub const PARAM_COUNT: usize = 44;
const NAME_MAX_LEN: usize = 20;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Program data out of range: {}", .0)]
    DataOutOfRange(usize),
    #[error("Program name out of range: {}", .0)]
    NameOutOfRange(usize),
}

#[derive(Debug)]
pub struct Program {
    bank: ProgramBank,
    nb: ProgramNumber,

    original_data: Cow<'static, [u8]>,
    original_name: Cow<'static, str>,

    data: Cow<'static, [u8]>,
    name: Cow<'static, str>,
}

impl Program {
    pub fn try_new(
        bank: ProgramBank,
        nb: ProgramNumber,
        data: Vec<u8>,
        name: String,
    ) -> Result<Self, Error> {
        if data.len() > PARAM_COUNT {
            return Err(Error::DataOutOfRange(data.len()));
        }

        if name.len() > NAME_MAX_LEN {
            return Err(Error::NameOutOfRange(name.len()));
        }

        let data = Cow::<[u8]>::from(data);
        let name = Cow::<str>::from(name);

        Ok(Program {
            bank,
            nb,
            original_data: data.clone(),
            original_name: name.clone(),
            data,
            name,
        })
    }

    pub fn bank(&self) -> ProgramBank {
        self.bank
    }

    pub fn nb(&self) -> ProgramNumber {
        self.nb
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn has_changed(&self) -> bool {
        matches!(self.data, Cow::Owned(_)) | matches!(self.name, Cow::Owned(_))
    }

    pub fn undo(&mut self) {
        self.data = self.original_data.clone();
        self.name = self.original_name.clone();
    }

    pub fn apply(&mut self) {
        if matches!(self.data, Cow::Owned(_)) {
            std::mem::swap(&mut self.original_data, &mut self.data);
            self.data = self.original_data.clone();
        }

        assert!(matches!(self.data, Cow::Borrowed(_)));

        if matches!(self.name, Cow::Owned(_)) {
            std::mem::swap(&mut self.original_name, &mut self.name);
            self.name = self.original_name.clone();
        }

        assert!(matches!(self.name, Cow::Borrowed(_)));
    }

    pub fn rename_as(&mut self, mut name: String) {
        // Truncate the name so as to comply with the device's limits
        let buf = name.as_bytes();
        if buf.len() > NAME_MAX_LEN {
            name = String::from_utf8_lossy(&buf[..NAME_MAX_LEN]).to_string();
        }

        if name == self.name {
            return;
        }

        if name == self.original_name {
            if matches!(self.name, Cow::Owned(_)) {
                // renaming as the original name
                self.name = self.original_name.clone();
            }

            return;
        }

        self.name = Cow::<str>::from(name);
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

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct ProgramNumber(u8);

impl ProgramNumber {
    pub const fn into_inner(self) -> u8 {
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

impl fmt::Display for ProgramNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&(self.0 + 1), f)
    }
}
