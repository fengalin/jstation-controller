use crate::jstation::{take_split_bytes_u8, BufferBuilder, ProcedureBuilder, ProcedureId};

#[derive(Debug)]
pub struct ToMessageResp {
    pub res: Result<u8, Error>,
}

impl ProcedureId for ToMessageResp {
    const ID: u8 = 0x7f;
    const VERSION: u8 = 1;
}

// FIXME remove if not needed
impl ProcedureBuilder for ToMessageResp {
    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        let (req_proc, code) = match self.res {
            Ok(req_proc) => (req_proc, 0),
            Err(err) => err.into(),
        };

        buffer.push_fixed_size_data([req_proc ,code].into_iter());
    }
}

impl ToMessageResp {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> nom::IResult<&'i [u8], ToMessageResp> {
        let (i, req_proc) = take_split_bytes_u8(input, checksum)?;
        let (i, code) = take_split_bytes_u8(i, checksum)?;

        Ok((i, ToMessageResp { res: try_from(req_proc, code) }))
    }
}


#[derive(Clone, Copy, Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown Procedure Id x{:02x}", .req_proc)]
    ProcedureId { req_proc: u8 },
    #[error("Invalid Procedure Version for x{:02x}", .req_proc)]
    ProcedureVersion { req_proc: u8 },
    #[error("Sysex Message Checksum Error for x{:02x}", .req_proc)]
    SysexChecksum { req_proc: u8 },
    #[error("Sysex Request Wrong Size for x{:02x}", .req_proc)]
    SysexSize { req_proc: u8 },
    #[error("MIDI Overrun Error for x{:02x}", .req_proc)]
    MIDIOverrun { req_proc: u8 },
    #[error("Invalid Program Number for x{:02x}", .req_proc)]
    ProgramNumber { req_proc: u8 },
    #[error("Invalid User Program Number for x{:02x}", .req_proc)]
    UserProgramNumber { req_proc: u8 },
    #[error("Invalid Bank Number for x{:02x}", .req_proc)]
    BankNumber { req_proc: u8 },
    #[error("Wrong Data Count for x{:02x}", .req_proc)]
    DataCount { req_proc: u8 },
    #[error("Unknown OS Command for x{:02x}", .req_proc)]
    OSCommand { req_proc: u8 },
    #[error("Wrong Mode for OS Command for x{:02x}", .req_proc)]
    OSCommandMode { req_proc: u8 },
    #[error("Unknown error {} for x{:02x}", .code, .req_proc)]
    Unknown { req_proc: u8, code: u8 },
}

// FIXME these should be generated from a single definition

impl From<Error> for (u8, u8) {
    fn from(err: Error) -> (u8, u8) {
        use Error::*;
        match err {
            ProcedureId { req_proc } => (req_proc, 1),
            ProcedureVersion { req_proc } => (req_proc, 2),
            SysexChecksum { req_proc } => (req_proc, 3),
            SysexSize { req_proc } => (req_proc, 4),
            MIDIOverrun { req_proc } => (req_proc, 5),
            ProgramNumber { req_proc } => (req_proc, 6),
            UserProgramNumber { req_proc } => (req_proc, 7),
            BankNumber { req_proc } => (req_proc, 8),
            DataCount { req_proc } => (req_proc, 9),
            OSCommand { req_proc } => (req_proc, 10),
            OSCommandMode { req_proc } => (req_proc, 11),
            Unknown { req_proc, code } => (req_proc, code),
        }
    }
}

fn try_from(req_proc: u8, code: u8) -> Result<u8, Error> {
    use Error::*;

    match code {
        0 => Ok(req_proc),
        1 => Err(ProcedureId { req_proc }),
        2 => Err(ProcedureVersion { req_proc }),
        3 => Err(SysexChecksum { req_proc }),
        4 => Err(SysexSize { req_proc }),
        5 => Err(MIDIOverrun { req_proc }),
        6 => Err(ProgramNumber { req_proc }),
        7 => Err(UserProgramNumber { req_proc }),
        8 => Err(BankNumber { req_proc }),
        9 => Err(DataCount { req_proc }),
        10 => Err(OSCommand { req_proc }),
        11 => Err(OSCommandMode { req_proc }),
        other => Err(Unknown { req_proc, code: other }),
    }
}
