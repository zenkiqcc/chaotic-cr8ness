use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum QrngError {
    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Device not initialized")]
    DeviceNotInitialized,
    #[error("Communication error: {0}")]
    CommunicationError(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("TLS error: {0}")]
    TlsError(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
} 