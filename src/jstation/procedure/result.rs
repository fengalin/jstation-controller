use std::fmt;

use crate::jstation::{
    split_bytes, take_split_bytes_u8, BufferBuilder, ProcedureBuilder
};

#[derive(Debug)]
pub struct ToMessageResp {
    pub req_proc: u8,
    pub code: u8,
}

impl ProcedureBuilder for ToMessageResp {
    const ID: u8 = 0x7f;
    const VERSION: u8 = 1;

    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        buffer.push_fixed_size_data(
            split_bytes::from_u8(self.req_proc).into_iter()
            .chain(split_bytes::from_u8(self.code).into_iter())
        );
    }
}

impl ToMessageResp {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> nom::IResult<&'i [u8], ToMessageResp> {
        let (i, req_proc) = take_split_bytes_u8(input, checksum)?;
        let (i, code) = take_split_bytes_u8(i, checksum)?;

        Ok((i, ToMessageResp { req_proc, code }))
    }
}

impl fmt::Display for ToMessageResp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Response to procedure x{:02x}: ", self.req_proc))?;

        let msg = match self.code {
            0 => "OK",
            1 => "Unknown Procedure Id",
            2 => "Invalid Procedure Version",
            3 => "Sysex Message Checksum Error",
            4 => "Sysex Request Wrong Size",
            5 => "MIDI Overrun Error",
            6 => "Invalid Program Number",
            7 => "Invalid User Program Number",
            8 => "Invalid Bank Number",
            9 => "Wrong Data Count",
           10 => "Unknown OS Command",
           11 => "Wrong Mode for OS Command",
           other => return f.write_fmt(format_args!("code {other}")),
        };

        f.write_str(msg)
    }
}
