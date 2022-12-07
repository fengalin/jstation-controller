use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("MIDI initialization failed")]
    Init(#[from] midir::InitError),

    #[error("Error connecting to MIDI port {}", .0)]
    Connection(Arc<str>),

    #[error("MIDI port not connected")]
    NotConnected,

    #[error("Midi port creation failed")]
    PortCreation,

    #[error("MIDI port connection failed")]
    PortConnection,

    #[error("Couldn't retrieve a MIDI port name")]
    PortInfoError(#[from] midir::PortInfoError),

    #[error("Invalid MIDI port name {}", .0)]
    PortNotFound(Arc<str>),

    #[error("MIDI port refresh discarded while scanning")]
    ScanningPorts,

    #[error("Couldn't send MIDI message: {}", .0)]
    Send(#[from] midir::SendError),
}
