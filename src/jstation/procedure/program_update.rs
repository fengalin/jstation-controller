use nom::IResult;

use crate::jstation::{take_split_bytes_u16, take_split_bytes_bool, ProcedureBuilder, data::ProgramData};

#[derive(Debug)]
pub struct ProgramUpdateReq;

impl ProcedureBuilder for ProgramUpdateReq {
    const ID: u8 = 0x60;
    const VERSION: u8 = 2;
}

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

impl ProgramUpdateResp {
    pub const ID: u8 = 0x61;
    pub const VERSION: u8 = 2;

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
