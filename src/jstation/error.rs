use std::sync::Arc;

use crate::midi;

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("Unknown J-Station CC number {}", .0)]
    CCNumberUnknown(u8),

    #[error("Inactive Parameter {} (discriminant {}), value {}", .param, .discriminant, .value)]
    ParameterInactive {
        param: String,
        discriminant: String,
        value: u8,
    },

    #[error("Parameter number {} out of range", .0)]
    ParameterNumberOutOfRange(u8),

    #[error("Value {} out of range: ({}..={})", .value, .min, .max)]
    ValueOutOfRange { value: u8, min: u8, max: u8 },

    #[error("Program number {} out of range", .0)]
    ProgramNumberOutOfRange(u8),

    #[error("Program data size {} out of range", .0)]
    ProgramDataOutOfRange(usize),

    #[error("Program name size {} out of range", .0)]
    ProgramNameOutOfRange(usize),

    #[error("Unknown Programs Bank {}", .0)]
    ProgramsBank(u8),

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
    pub fn with_context<E>(ctx: impl Into<Arc<str>>, source: E) -> Self
    where
        E: 'static + std::error::Error + Send + Sync,
    {
        Error::WithContext {
            ctx: ctx.into(),
            source: Arc::new(source),
        }
    }

    pub fn is_handshake_timeout(&self) -> bool {
        matches!(self, Error::HandshakeTimeout)
    }

    pub fn is_unknown_cc(&self) -> bool {
        matches!(self, Error::CCNumberUnknown(_))
    }

    pub fn is_inactive_param(&self) -> bool {
        matches!(self, Error::ParameterInactive { .. })
    }
}
