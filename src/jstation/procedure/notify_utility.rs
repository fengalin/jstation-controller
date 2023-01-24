use crate::jstation::{ProcedureBuilder, ProcedureId};

#[derive(Debug)]
pub struct NotifyUtility;

impl ProcedureId for NotifyUtility {
    const ID: u8 = 0x23;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for NotifyUtility {}

impl NotifyUtility {
    pub fn parse<'i>(input: &'i [u8], _checksum: &mut u8) -> nom::IResult<&'i [u8], NotifyUtility> {
        Ok((input, NotifyUtility))
    }
}
