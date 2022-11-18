use nom::IResult;

use crate::{
    jstation::{take_split_bytes_bool, take_split_bytes_chan, take_split_bytes_len, BufferBuilder, ProcedureBuilder},
    midi,
};

#[derive(Debug)]
pub struct WhoAmIReq {
    pub resp_on_chan: bool,
}

impl Default for WhoAmIReq {
    fn default() -> Self {
        WhoAmIReq { resp_on_chan: true }
    }
}

impl ProcedureBuilder for WhoAmIReq {
    const ID: u8 = 0x40;
    const VERSION: u8 = 1;

    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        buffer.push_fixed_size_data(std::iter::once(self.resp_on_chan.into()));
    }
}

impl WhoAmIReq {
    pub fn parse<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], WhoAmIReq> {
        let (i, resp_on_chan) = take_split_bytes_bool(i, checksum)?;

        Ok((i, WhoAmIReq { resp_on_chan }))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WhoAmIResp {
    pub receive_chan: midi::Channel,
    pub transmit_chan: midi::Channel,
    pub sysex_chan: midi::Channel,
}

impl WhoAmIResp {
    pub const ID: u8 = 0x41;
    pub const VERSION: u8 = 1;

    pub fn parse<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], WhoAmIResp> {
        let (i, _) = take_split_bytes_len(i, checksum, 3)?;

        let (i, receive_chan) = take_split_bytes_chan(i, checksum)?;
        let (i, transmit_chan) = take_split_bytes_chan(i, checksum)?;
        let (i, sysex_chan) = take_split_bytes_chan(i, checksum)?;

        Ok((i, WhoAmIResp {
            receive_chan,
            transmit_chan,
            sysex_chan,
        }))
    }
}
