use nom::IResult;

use crate::jstation::ProcedureBuilder;

#[derive(Debug)]
pub struct ReloadProgramReq;

impl ProcedureBuilder for ReloadProgramReq {
    const ID: u8 = 0x20;
    const VERSION: u8 = 1;
}

impl ReloadProgramReq {
    pub fn parse<'i>(input: &'i [u8], _checksum: &mut u8) -> IResult<&'i [u8], ReloadProgramReq> {
        Ok((input, ReloadProgramReq))
    }
}
