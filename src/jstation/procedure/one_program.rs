use nom::{error::{self, Error}, IResult};

use crate::jstation::{
    data::{Program, ProgramBank, ProgramData, ProgramNumber},
    split_bytes, take_split_bytes_u16, take_split_bytes_u8, BufferBuilder, ProcedureBuilder
};

#[derive(Debug)]
pub struct OneProgramReq {
    pub bank: ProgramBank,
    pub nb: ProgramNumber,
}

impl ProcedureBuilder for OneProgramReq {
    const ID: u8 = 0x01;
    const VERSION: u8 = 1;

    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        buffer.push_fixed_size_data(
            split_bytes::from_u8(self.bank.into()).into_iter()
            .chain(split_bytes::from_u8(self.nb.into()).into_iter())
        );
    }
}

impl OneProgramReq {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], OneProgramReq> {
        let (i, bank) = take_split_bytes_u8(input, checksum)?;
        let bank = ProgramBank::from(bank);

        let (i, nb) = take_split_bytes_u8(i, checksum)?;
        let nb = ProgramNumber::from(nb);

        Ok((i, OneProgramReq { bank, nb }))
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
            split_bytes::from_u8(self.prog.bank().into()).into_iter()
            .chain(split_bytes::from_u8(self.prog.nb().into()).into_iter())
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
        let (i, bank) = take_split_bytes_u8(input, checksum)?;
        let bank = ProgramBank::from(bank);

        let (i, nb) = take_split_bytes_u8(i, checksum)?;
        let nb = ProgramNumber::from(nb);

        let (i, len) = take_split_bytes_u16(i, checksum)?;

        let (i, prog_data) = ProgramData::parse(i, checksum, len)
            .map_err(|err| {
                log::error!("OneProgramResp: {err}");

                nom::Err::Failure(Error::new(
                    input,
                    error::ErrorKind::LengthValue,
                ))
            })?;

        Ok((i, OneProgramResp { prog: Program::new(bank, nb, prog_data) }))
    }
}
