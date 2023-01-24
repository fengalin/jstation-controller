use nom::IResult;
use smallvec::SmallVec;

use crate::jstation::{split_bytes, take_split_bytes_u16, take_split_bytes_u8, BufferBuilder, ProcedureBuilder, ProcedureId, ProgramNb};

#[derive(Debug)]
pub struct ProgramIndicesReq;

impl ProcedureId for ProgramIndicesReq {
    const ID: u8 = 0x13;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for ProgramIndicesReq {}

impl ProgramIndicesReq {
    pub fn parse<'i>(input: &'i [u8], _checksum: &mut u8) -> IResult<&'i [u8], ProgramIndicesReq> {
        Ok((input, ProgramIndicesReq))
    }
}

const DEFAULT_INDICES_LEN: usize = 32;

#[derive(Debug, Default)]
pub struct ProgramIndicesResp {
    pub numbers: SmallVec::<[ProgramNb; DEFAULT_INDICES_LEN]>,
}

impl ProcedureId for ProgramIndicesResp {
    const ID: u8 = 0x14;
    const VERSION: u8 = 1;
}

impl ProcedureBuilder for ProgramIndicesResp {
    fn push_variable_size_data(&self, buffer: &mut BufferBuilder) {
        let buf = Vec::from_iter(
            self.numbers.iter().cloned().flat_map(|nb| split_bytes::from_u8(nb.into()))
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

            let nb = ProgramNb::try_from(indice).map_err(|err| {
                log::error!("ProgramIndicesResp: {err}");

                nom::Err::Failure(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::TooLarge,
                ))
            })?;

            prg_indices.numbers.push(nb);
        }

        Ok((i, prg_indices))
    }
}
