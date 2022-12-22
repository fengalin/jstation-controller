use nom::IResult;
use smallvec::SmallVec;

use crate::jstation::{split_bytes, take_split_bytes_u16, take_split_bytes_u8, BufferBuilder, ProcedureBuilder};

#[derive(Debug)]
pub struct ProgramIndicesReq;

impl ProcedureBuilder for ProgramIndicesReq {
    const ID: u8 = 0x13;
    const VERSION: u8 = 1;
}

impl ProgramIndicesReq {
    pub fn parse<'i>(input: &'i [u8], _checksum: &mut u8) -> IResult<&'i [u8], ProgramIndicesReq> {
        Ok((input, ProgramIndicesReq))
    }
}

const DEFAULT_INDICES_LEN: usize = 32;

#[derive(Debug, Default)]
pub struct ProgramIndicesResp {
    pub indices: SmallVec::<[u8; DEFAULT_INDICES_LEN]>,
}

impl ProcedureBuilder for ProgramIndicesResp {
    const ID: u8 = 0x14;
    const VERSION: u8 = 1;

    fn push_variable_size_data(&self, buffer: &mut BufferBuilder) {
        let buf = Vec::from_iter(
            self.indices.iter().cloned().flat_map(split_bytes::from_u8)
        );
        buffer.push_variable_size_data(buf.into_iter());
    }
}

impl ProgramIndicesResp {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], ProgramIndicesResp> {
        let (mut i, len) = take_split_bytes_u16(input, checksum)?;

        let mut prg_indices = ProgramIndicesResp::default();
        for _ in 0..len {
            let (i_, indice) = take_split_bytes_u8(i, checksum)?;
            i = i_;
            prg_indices.indices.push(indice);
        }

        Ok((i, prg_indices))
    }
}
