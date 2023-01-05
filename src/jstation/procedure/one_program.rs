use nom::{error::{self, Error}, IResult};

use crate::jstation::{
    data::{Program, ProgramData, ProgramId},
    split_bytes, take_split_bytes_u16, take_split_bytes_u8, take_u8, BufferBuilder, ProcedureBuilder
};

#[derive(Debug)]
pub struct OneProgramReq {
    pub id: ProgramId,
}

impl ProcedureBuilder for OneProgramReq {
    const ID: u8 = 0x01;
    const VERSION: u8 = 1;

    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        buffer.push_fixed_size_data(
            [self.id.progs_bank().into(), self.id.nb().into()].into_iter()
        );
    }
}

impl OneProgramReq {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], OneProgramReq> {
        let (i, progs_bank) = take_u8(input, checksum)?;
        let (i, nb) = take_u8(i, checksum)?;

        let id = ProgramId::try_from_raw(progs_bank, nb).map_err(|err| {
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

impl ProcedureBuilder for OneProgramResp {
    const ID: u8 = 0x02;
    const VERSION: u8 = 1;

    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        buffer.push_fixed_size_data(
            split_bytes::from_u8(self.prog.id().progs_bank().into()).into_iter()
            .chain(split_bytes::from_u8(self.prog.id().nb().into()).into_iter())
        );
    }

    fn push_variable_size_data(&self, buffer: &mut crate::jstation::sysex::BufferBuilder) {
        let mut buf: Vec<u8> = self.prog.data().iter().map(Into::into).collect();
        buf.extend(self.prog.name().as_bytes());
        // Terminal 0 for name
        buf.push(0x00);

        buffer.push_variable_size_data(buf.into_iter());
    }
}

impl OneProgramResp {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], OneProgramResp> {
        let (i, progs_bank) = take_split_bytes_u8(input, checksum)?;
        let (i, nb) = take_split_bytes_u8(i, checksum)?;

        let id = ProgramId::try_from_raw(progs_bank, nb).map_err(|err| {
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
