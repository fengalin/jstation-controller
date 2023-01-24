use nom::{error::{self, Error}, IResult};

use crate::jstation::{
    data::{Program, ProgramData, ProgramId},
    take_split_bytes_u16, take_split_bytes_u8, take_u8, BufferBuilder, ProcedureBuilder, ProcedureId,
};

#[derive(Debug)]
pub struct OneProgramReq {
    pub id: ProgramId,
}

impl ProcedureId for OneProgramReq {
    const ID: u8 = 0x01;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for OneProgramReq {
    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        buffer.push_fixed_size_data([
            self.id.bank().into(),
            self.id.nb().into(),
        ].into_iter());
    }
}

impl OneProgramReq {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], OneProgramReq> {
        let (i, bank) = take_u8(input, checksum)?;
        let (i, nb) = take_u8(i, checksum)?;

        let id = ProgramId::try_from_raw(bank, nb).map_err(|err| {
            log::error!("OneProgramReq: {err}");

            nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TooLarge,
            ))
        })?;

        Ok((i, OneProgramReq { id }))
    }
}

#[derive(Debug)]
pub struct OneProgramResp {
    pub prog: Program,
}

impl OneProgramResp {
    pub fn from(prog: &Program) -> OneProgramRefResp<'_> {
        OneProgramRefResp(prog)
    }
}

impl ProcedureId for OneProgramResp {
    const ID: u8 = 0x02;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for OneProgramResp {
    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        OneProgramRefResp(&self.prog).push_fixed_size_data(buffer)
    }

    fn push_variable_size_data(&self, buffer: &mut BufferBuilder) {
        OneProgramRefResp(&self.prog).push_variable_size_data(buffer)
    }
}

#[derive(Debug)]
pub struct OneProgramRefResp<'a>(pub &'a Program);

impl<'a> ProcedureId for OneProgramRefResp<'a> {
    const ID: u8 = OneProgramResp::ID;
    const VERSION: u8 = OneProgramResp::VERSION;
}

impl<'a> ProcedureBuilder for OneProgramRefResp<'a> {
    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        buffer.push_fixed_size_data([
            self.0.id().bank().into(),
            self.0.id().nb().into(),
        ].into_iter());
    }

    fn push_variable_size_data(&self, buffer: &mut BufferBuilder) {
        let buf = Vec::<u8>::from_iter(self.0.data().serialize());
        buffer.push_variable_size_data(buf.into_iter());
    }
}

impl OneProgramResp {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], OneProgramResp> {
        let (i, bank) = take_split_bytes_u8(input, checksum)?;
        let (i, nb) = take_split_bytes_u8(i, checksum)?;

        let id = ProgramId::try_from_raw(bank, nb).map_err(|err| {
            log::error!("OneProgramResp: {err}");

            nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TooLarge,
            ))
        })?;

        let (i, len) = take_split_bytes_u16(i, checksum)?;

        let (i, prog_data) = ProgramData::parse(i, checksum, len)
            .map_err(|err| {
                log::error!("OneProgramResp: {err}");

                nom::Err::Failure(Error::new(
                    input,
                    error::ErrorKind::LengthValue,
                ))
            })?;

        Ok((i, OneProgramResp { prog: Program::new(id, prog_data) }))
    }
}
