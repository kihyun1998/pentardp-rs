use std::io::{Read, Write};
use thiserror::Error;

// 단일 파일 모듈
pub mod tpkt;

// 디렉토리 모듈
pub mod x224;

/// PDU 파싱 및 직렬화 결과 타입
pub type Result<T> = std::result::Result<T, PduError>;

/// PDU 관련 에러 타입
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

/// PDU 공통 인터페이스
pub trait Pdu: Sized {
    /// PDU를 바이트 스트림으로 인코딩
    fn encode(&self, buffer: &mut dyn Write) -> Result<()>;

    /// 바이트 스트림에서 PDU를 디코딩
    fn decode(buffer: &mut dyn Read) -> Result<Self>;

    /// PDU의 전체 크기 (바이트)
    fn size(&self) -> usize;
}

/// 헤더가 있는 PDU
pub trait PduWithHeader: Pdu {
    type Header;

    /// PDU의 헤더 참조 반환
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
