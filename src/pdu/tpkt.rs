use crate::pdu::{Pdu, PduError, PduWithHeader, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// TPKT 프로토콜 버전
pub const TPKT_VERSION: u8 = 0x03;

/// TPKT 헤더 크기 (바이트)
pub const TPKT_HEADER_SIZE: usize = 4;

/// TPKT 헤더 구조
///
/// RFC 1006에 정의된 TPKT 헤더
/// ```text
/// +--------+--------+--------+--------+
/// |Version |Reserved|     Length      |
/// | (0x03) | (0x00) |   (Big-Endian)  |
/// +--------+--------+--------+--------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpktHeader {
    /// TPKT 버전 (항상 0x03)
    pub version: u8,
    /// 예약 필드 (항상 0x00)
    pub reserved: u8,
    /// 전체 패킷 길이 (헤더 + 페이로드)
    pub length: u16,
}

impl TpktHeader {
    /// 새로운 TPKT 헤더 생성
    pub fn new(payload_length: u16) -> Self {
        Self {
            version: TPKT_VERSION,
            reserved: 0,
            length: TPKT_HEADER_SIZE as u16 + payload_length,
        }
    }

    /// 헤더 인코딩
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u8(self.version)?;
        buffer.write_u8(self.reserved)?;
        buffer.write_u16::<BigEndian>(self.length)?;
        Ok(())
    }

    /// 헤더 디코딩
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let version = buffer.read_u8()?;
        let reserved = buffer.read_u8()?;
        let length = buffer.read_u16::<BigEndian>()?;

        // 버전 검증
        if version != TPKT_VERSION {
            return Err(PduError::UnsupportedVersion(version));
        }

        // 길이 검증 (최소 헤더 크기 이상이어야 함)
        if (length as usize) < TPKT_HEADER_SIZE {
            return Err(PduError::InvalidLength {
                expected: TPKT_HEADER_SIZE,
                actual: length as usize,
            });
        }

        Ok(Self {
            version,
            reserved,
            length,
        })
    }

    /// 페이로드 길이 반환
    pub fn payload_length(&self) -> usize {
        (self.length as usize).saturating_sub(TPKT_HEADER_SIZE)
    }
}

/// TPKT 패킷
///
/// TPKT 헤더와 페이로드로 구성
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpktPacket {
    /// TPKT 헤더
    header: TpktHeader,
    /// 페이로드 (X.224 데이터)
    payload: Vec<u8>,
}

impl TpktPacket {
    /// 새로운 TPKT 패킷 생성
    pub fn new(payload: Vec<u8>) -> Self {
        let header = TpktHeader::new(payload.len() as u16);
        Self { header, payload }
    }

    /// 페이로드 참조 반환
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// 페이로드 소유권 이전
    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }

    /// 페이로드 가변 참조 반환
    pub fn payload_mut(&mut self) -> &mut Vec<u8> {
        &mut self.payload
    }
}

impl Pdu for TpktPacket {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        self.header.encode(buffer)?;
        buffer.write_all(&self.payload)?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let header = TpktHeader::decode(buffer)?;
        let payload_len = header.payload_length();

        let mut payload = vec![0u8; payload_len];
        buffer.read_exact(&mut payload)?;

        Ok(Self { header, payload })
    }

    fn size(&self) -> usize {
        self.header.length as usize
    }
}

impl PduWithHeader for TpktPacket {
    type Header = TpktHeader;

    fn header(&self) -> &Self::Header {
        &self.header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_tpkt_header_new() {
        let header = TpktHeader::new(10);
        assert_eq!(header.version, TPKT_VERSION);
        assert_eq!(header.reserved, 0);
        assert_eq!(header.length, 14); // 4 (header) + 10 (payload)
        assert_eq!(header.payload_length(), 10);
    }

    #[test]
    fn test_tpkt_header_encode_decode() {
        let original = TpktHeader::new(100);
        let mut buffer = Vec::new();
        original.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), TPKT_HEADER_SIZE);
        assert_eq!(buffer[0], TPKT_VERSION);
        assert_eq!(buffer[1], 0);

        let mut cursor = Cursor::new(buffer);
        let decoded = TpktHeader::decode(&mut cursor).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_tpkt_header_invalid_version() {
        let buffer = vec![0x02, 0x00, 0x00, 0x04]; // 잘못된 버전
        let mut cursor = Cursor::new(buffer);
        let result = TpktHeader::decode(&mut cursor);

        assert!(matches!(result, Err(PduError::UnsupportedVersion(0x02))));
    }

    #[test]
    fn test_tpkt_header_invalid_length() {
        let buffer = vec![0x03, 0x00, 0x00, 0x02]; // 너무 작은 길이
        let mut cursor = Cursor::new(buffer);
        let result = TpktHeader::decode(&mut cursor);

        assert!(matches!(
            result,
            Err(PduError::InvalidLength {
                expected: 4,
                actual: 2
            })
        ));
    }

    #[test]
    fn test_tpkt_packet_new() {
        let payload = vec![1, 2, 3, 4, 5];
        let packet = TpktPacket::new(payload.clone());

        assert_eq!(packet.payload(), &payload[..]);
        assert_eq!(packet.size(), 9); // 4 (header) + 5 (payload)
    }

    #[test]
    fn test_tpkt_packet_encode_decode() {
        let payload = vec![0x11, 0x22, 0x33, 0x44];
        let original = TpktPacket::new(payload);

        let mut buffer = Vec::new();
        original.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), 8); // 4 (header) + 4 (payload)

        let mut cursor = Cursor::new(buffer);
        let decoded = TpktPacket::decode(&mut cursor).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_tpkt_packet_roundtrip() {
        let test_cases = vec![
            vec![],
            vec![0x00],
            vec![0x01, 0x02, 0x03],
            vec![0xFF; 100],
        ];

        for payload in test_cases {
            let original = TpktPacket::new(payload);
            let mut buffer = Vec::new();
            original.encode(&mut buffer).unwrap();

            let mut cursor = Cursor::new(buffer);
            let decoded = TpktPacket::decode(&mut cursor).unwrap();

            assert_eq!(original, decoded);
        }
    }
}
