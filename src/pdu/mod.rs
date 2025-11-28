use std::io::{Read, Write};
use thiserror::Error;

pub mod mcs;
pub mod rdp;
pub mod tpkt;
pub mod x224;

/// PDU parsing and serialization result type
pub type Result<T> = std::result::Result<T, PduError>;

/// PDU related error types
#[derive(Error, Debug)]
pub enum PduError {
    #[error("Invalid length: expected {expected}, got {actual}")]
    InvalidLength { expected: usize, actual: usize },

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(u8),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Insufficient data: need {needed} bytes, got {available}")]
    InsufficientData { needed: usize, available: usize },

    #[error("Invalid PDU type: {0:#x}")]
    InvalidPduType(u8),
}

/// PDU common interface
pub trait Pdu: Sized {
    /// Encode PDU to byte stream
    fn encode(&self, buffer: &mut dyn Write) -> Result<()>;

    /// Decode PDU from byte stream
    fn decode(buffer: &mut dyn Read) -> Result<Self>;

    /// Total size of PDU (in bytes)
    fn size(&self) -> usize;
}

/// PDU with header
pub trait PduWithHeader: Pdu {
    type Header;

    /// Return reference to PDU header
    fn header(&self) -> &Self::Header;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = PduError::InvalidLength {
            expected: 10,
            actual: 5,
        };
        assert_eq!(err.to_string(), "Invalid length: expected 10, got 5");

        let err = PduError::UnsupportedVersion(0x05);
        assert_eq!(err.to_string(), "Unsupported version: 5");
    }
}
