use nom::IResult;

use crate::jstation::{take_split_bytes_u16, take_split_bytes_bool, BufferBuilder, ProcedureBuilder, ProcedureId, data::ProgramData};

#[derive(Debug)]
pub struct ProgramUpdateReq;

impl ProcedureId for ProgramUpdateReq {
    const ID: u8 = 0x60;
    const VERSION: u8 = 2;
}

impl ProcedureBuilder for ProgramUpdateReq {}

impl ProgramUpdateReq {
    pub fn parse<'i>(i: &'i [u8], _checksum: &mut u8) -> IResult<&'i [u8], ProgramUpdateReq> {
        Ok((i, ProgramUpdateReq))
    }
}

#[derive(Debug)]
pub struct ProgramUpdateResp {
    pub has_changed: bool,
    pub prog_data: ProgramData,
}

impl ProcedureId for ProgramUpdateResp {
    const ID: u8 = 0x61;
    const VERSION: u8 = 2;
}

impl ProgramUpdateResp {
    pub fn from_changed(prog_data: &ProgramData) -> ProgramUpdateRefResp<'_> {
        ProgramUpdateRefResp {
            has_changed: true,
            prog_data,
        }
    }

    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], ProgramUpdateResp> {
        let (i, mut len) = take_split_bytes_u16(input, checksum)?;

        let (i, has_changed) = take_split_bytes_bool(i, checksum)?;
        len -= 1;

        let (i, prog_data) = ProgramData::parse(i, checksum, len)
            .map_err(|err| {
                log::error!("ProgramUpdateResp: {err}");

                nom::Err::Failure(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::LengthValue,
                ))
            })?;

        Ok((i, ProgramUpdateResp { has_changed, prog_data } ))
    }
}

#[derive(Debug)]
pub struct ProgramUpdateRefResp<'a> {
    pub has_changed: bool,
    pub prog_data: &'a ProgramData,
}

impl<'a> ProcedureId for ProgramUpdateRefResp<'a> {
    const ID: u8 = ProgramUpdateResp::ID;
    const VERSION: u8 = ProgramUpdateResp::VERSION;
}

impl<'a> ProcedureBuilder for ProgramUpdateRefResp<'a> {
    fn push_variable_size_data(&self, buffer: &mut BufferBuilder) {
        let mut buf = vec![u8::from(self.has_changed)];
        buf.extend(self.prog_data.serialize());

        buffer.push_variable_size_data(buf.into_iter());
    }
}
