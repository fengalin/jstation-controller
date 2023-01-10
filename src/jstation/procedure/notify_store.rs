use crate::jstation::{
    data::ProgramNb,
    take_split_bytes_u8, BufferBuilder, ProcedureBuilder
};

#[derive(Debug)]
pub struct NotifyStore {
    pub nb: ProgramNb,
}

impl ProcedureBuilder for NotifyStore {
    const ID: u8 = 0x22;
    const VERSION: u8 = 1;

    fn push_fixed_size_data(&self, buffer: &mut BufferBuilder) {
        buffer.push_fixed_size_data(std::iter::once(self.nb.into()));
    }
}

impl NotifyStore {
    pub fn parse<'i>(input: &'i [u8], checksum: &mut u8) -> nom::IResult<&'i [u8], NotifyStore> {
        let (i, nb) = take_split_bytes_u8(input, checksum)?;

        let nb = ProgramNb::try_from(nb).map_err(|err| {
            log::error!("NotifyStore: {err}");

            nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TooLarge,
            ))
        })?;

        Ok((i, NotifyStore { nb }))
    }
}
