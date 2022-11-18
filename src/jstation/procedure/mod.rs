use nom::{
    bytes::complete::take,
    error::{self, Error},
    IResult,
};

use crate::{
    jstation::{split_bytes, sysex},
    midi,
};

//pub const MERGE_RESPONSE: u8 = 0x7f;

pub trait ProcedureBuilder {
    const ID: u8;
    const VERSION: u8;

    fn push_fixed_size_data(&self, _buffer: &mut sysex::BufferBuilder) {}
    fn push_variable_size_data(&self, _buffer: &mut sysex::BufferBuilder) {}

    fn build_for(&self, chan: midi::Channel) -> Vec<u8> {
        let mut buffer = sysex::BufferBuilder::new(chan, Self::ID, Self::VERSION);

        self.push_fixed_size_data(&mut buffer);
        self.push_variable_size_data(&mut buffer);

        buffer.build()
    }
}

macro_rules! declare_procs(
    ( $( $module:ident: $( $proc:ident $(,)? )*; )* ) => {
        $(
            pub mod $module;
            pub use $module::{$( $proc, )*};
        )*

        #[derive(Debug)]
        pub enum Procedure {
            $(
                $( $proc($proc), )*
            )*
        }

        $(
            $(
                impl From<$proc> for Procedure {
                    fn from(proc: $proc) -> Self {
                        Procedure::$proc(proc)
                    }
                }
            )*
        )*

        pub fn parse<'i>(i: &'i [u8], checksum: &mut u8) -> IResult<&'i [u8], Procedure> {
            let (i, proc_id_version) = take(3usize)(i)?;

            *checksum = *checksum ^ proc_id_version[0] ^ proc_id_version[1] ^ proc_id_version[2];

            let proc_id = proc_id_version[0];
            let version = split_bytes::to_u8(&proc_id_version[1..3]);

            match (proc_id, version) {
                $($(
                    ($module::$proc::ID, $module::$proc::VERSION) => {
                        $module::$proc::parse(i, checksum)
                            .map(|(i, proc)| (i, proc.into()))
                            .map_err(|err| {
                                log::error!(
                                    "Failed to parse Procedure {}: {err}",
                                    stringify!($proc),
                                );

                                err
                            })
                    }
                )*)*
                _ => {
                    log::warn!("Unknown Procedure with id: 0x{proc_id:02x}, version: {version}");
                    Err(nom::Err::Failure(Error::new(i, error::ErrorKind::NoneOf)))
                }
            }
        }
    };
);

declare_procs!(
    who_am_i: WhoAmIReq, WhoAmIResp;
    utility_settings: UtilitySettingsReq, UtilitySettingsResp;
);
