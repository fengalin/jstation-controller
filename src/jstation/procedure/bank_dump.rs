use nom::IResult;

use crate::jstation::{split_bytes, take_split_bytes_u16, BufferBuilder, ProcedureBuilder, ProcedureId};

#[derive(Debug)]
pub struct BankDumpReq;

impl ProcedureId for BankDumpReq {
    const ID: u8 = 0x24;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for BankDumpReq {}

impl BankDumpReq {
    pub fn parse<'i>(input: &'i [u8], _checksum: &mut u8) -> IResult<&'i [u8], BankDumpReq> {
        Ok((input, BankDumpReq))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct StartBankDumpResp {
    pub total_len: u16,
}

impl ProcedureId for StartBankDumpResp {
    const ID: u8 = 0x25;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for StartBankDumpResp {
    fn push_variable_size_data(&self, buffer: &mut BufferBuilder) {
        buffer.push_variable_size_data(split_bytes::from_u16(self.total_len).into_iter());
    }
}

impl StartBankDumpResp {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], StartBankDumpResp> {
        let (i, total_len_size) = take_split_bytes_u16(input, checksum)?;
        if total_len_size > 2 {
            // Quoting gstation-edit:
            // > For some reasons, data len in a bank export from J-Edit
            // > is 4 when the actual size to read is 2 just like
            // > for the regular StartBankDumpResponse sent by the J-Station
            log::debug!("Ingoring incorrect StartBankDumpResp size");
        }

        let (i, total_len) = take_split_bytes_u16(i, checksum)?;

        Ok((i, StartBankDumpResp { total_len }))
    }
}

#[derive(Debug, Default)]
pub struct EndBankDumpResp;

impl ProcedureId for EndBankDumpResp {
    const ID: u8 = 0x26;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for EndBankDumpResp {}

impl EndBankDumpResp {
    pub fn parse<'i>(input: &'i [u8], _checksum: &mut u8) -> IResult<&'i [u8], EndBankDumpResp> {
        Ok((input, EndBankDumpResp))
    }
}
