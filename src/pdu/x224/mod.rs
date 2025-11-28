pub mod data;

pub use data::{DataHeader, DataPdu};

/// X.224 PDU 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PduType {
    /// Connection Request (CR)
    ConnectionRequest = 0xE0,
    /// Connection Confirm (CC)
    ConnectionConfirm = 0xD0,
    /// Disconnect Request (DR)
    DisconnectRequest = 0x80,
    /// Data Transfer (DT)
    Data = 0xF0,
    /// Error (ERR)
    Error = 0x70,
}

impl PduType {
    /// u8에서 PduType으로 변환 (EOT 플래그 무시)
    pub fn from_u8(value: u8) -> Option<Self> {
        // EOT 플래그를 제거하기 위해 0xFE로 마스킹
        match value & 0xFE {
            0xE0 => Some(Self::ConnectionRequest),
            0xD0 => Some(Self::ConnectionConfirm),
            0x80 => Some(Self::DisconnectRequest),
            0xF0 => Some(Self::Data),
            0x70 => Some(Self::Error),
            _ => None,
        }
    }
}
