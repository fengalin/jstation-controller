use std::sync::Arc;

use crate::{jstation::data::RawValue, midi};

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown J-Station CC number x{:02x}", .0)]
    CCNumber(u8),

    #[error("Parameter number {} out of range", .0)]
    ParameterNumberOutOfRange(u8),

    #[error("Parameter raw value {} out of range: ({}..={})", .value, .min, .max)]
    ParameterRawValueOutOfRange {
        value: RawValue,
        min: RawValue,
        max: RawValue,
    },

    #[error("Program number {} out of range", .0)]
    ProgramNumberOutOfRange(u8),

    #[error("Program data size {} out of range", .0)]
    ProgramDataOutOfRange(usize),

    #[error("Program name size {} out of range", .0)]
    ProgramNameOutOfRange(usize),

    #[error("Error Parsing MIDI message")]
    Parse,

    #[error("{}", .0)]
    Midi(#[from] midi::Error),

    #[error("No MIDI connection")]
    MidiNotConnected,

    #[error("An error occured sending a MIDI message")]
    MidiSend,

    #[error("Normal out of range: {}", .0)]
    NormalOutOfRange(f32),

    #[error("Device handshake timed out")]
    HandshakeTimeout,

    #[error("{}: {}", ctx, source)]
    WithContext {
        ctx: Arc<str>,
        source: Arc<dyn std::error::Error + Send + Sync>,
    },
}

impl Error {
    // FIXME there must be a better way...
    pub fn with_context<E>(ctx: &'static str, source: E) -> Self
    where
        E: 'static + std::error::Error + Send + Sync,
    {
        Error::WithContext {
            ctx: Arc::from(ctx),
            source: Arc::new(source),
        }
    }

    pub fn is_handshake_timeout(&self) -> bool {
        matches!(self, Error::HandshakeTimeout)
    }
}
