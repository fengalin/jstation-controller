use crate::jstation::ProcedureBuilder;

#[derive(Debug)]
pub struct NotifyUtility;

impl ProcedureBuilder for NotifyUtility {
    const ID: u8 = 0x23;
    const VERSION: u8 = 1;
}

impl NotifyUtility {
    pub fn parse<'i>(input: &'i [u8], _checksum: &mut u8) -> nom::IResult<&'i [u8], NotifyUtility> {
        Ok((input, NotifyUtility))
    }
}
