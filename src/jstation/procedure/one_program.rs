use nom::{error::{self, Error}, IResult};

use crate::jstation::{
    data::{Program, ProgramBank, ProgramNumber, RawValue},
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
    pub fn parse<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], OneProgramReq> {
        let (i, bank) = take_split_bytes_u8(i, checksum)?;
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
    pub fn parse<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], OneProgramResp> {
        let (i, bank) = take_split_bytes_u8(i, checksum)?;
        let bank = ProgramBank::from(bank);

        let (i, nb) = take_split_bytes_u8(i, checksum)?;
        let nb = ProgramNumber::from(nb);

        let (mut i, mut len) = take_split_bytes_u16(i, checksum)?;

        let mut data = Vec::<RawValue>::new();
        for _ in 0..Program::PARAM_COUNT {
            let (i_, byte) = take_split_bytes_u8(i, checksum)?;
            i = i_;
            data.push(byte.into());
        }

        len -= Program::PARAM_COUNT as u16;

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
        let prog = Program::try_new(bank, nb, data, name)
            .map_err(|err| {
                log::error!("{err}");

                nom::Err::Failure(Error::new(
                    i,
                    error::ErrorKind::LengthValue,
                ))
            })?;

        Ok((i, OneProgramResp { prog }))
    }
}
