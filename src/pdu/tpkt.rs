use crate::pdu::{Pdu, PduError, PduWithHeader, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// TPKT protocol version
pub const TPKT_VERSION: u8 = 0x03;

/// TPKT header size (bytes)
pub const TPKT_HEADER_SIZE: usize = 4;

/// TPKT Header Structure
///
// TPKT Header Defined in RFC 1006
/// ```text
/// +--------+--------+--------+--------+
/// |Version |Reserved| Length |
/// | (0x03) | (0x00) | (Big-Endian) |
/// +--------+--------+--------+--------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpktHeader {
    /// TPKT version (always 0x03)
    pub version: u8,
    /// Reserved field (always 0x00)
    pub reserved: u8,
    /// Total packet length (header + payload)
    pub length: u16,
}

impl TpktHeader {
    /// Create a new TPKT header
    pub fn new(payload_length: u16) -> Self {
        Self {
            version: TPKT_VERSION,
            reserved: 0,
            length: TPKT_HEADER_SIZE as u16 + payload_length,
        }
    }

    /// Header encoding
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u8(self.version)?;
        buffer.write_u8(self.reserved)?;
        buffer.write_u16::<BigEndian>(self.length)?;
        Ok(())
    }

    /// Header decoding
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let version = buffer.read_u8()?;
        let reserved = buffer.read_u8()?;
        let length = buffer.read_u16::<BigEndian>()?;

        // Version verification
        if version != TPKT_VERSION {
            return Err(PduError::UnsupportedVersion(version));
        }

        // Length validation (must be greater than or equal to the minimum header size)
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

    /// Return payload length
    pub fn payload_length(&self) -> usize {
        (self.length as usize).saturating_sub(TPKT_HEADER_SIZE)
    }
}

/// TPKT packet
///
/// Consists of a TPKT header and payload
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpktPacket {
    /// TPKT header
    header: TpktHeader,
    /// Payload (X.224 data)
    payload: Vec<u8>,
}

impl TpktPacket {
    /// Create a new TPKT packet
    pub fn new(payload: Vec<u8>) -> Self {
        let header = TpktHeader::new(payload.len() as u16);
        Self { header, payload }
    }

    /// Return a payload reference
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Transferring payload ownership
    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }

    /// Return a mutable reference to the payload
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
        let buffer = vec![0x02, 0x00, 0x00, 0x04]; // Wrong Version
        let mut cursor = Cursor::new(buffer);
        let result = TpktHeader::decode(&mut cursor);

        assert!(matches!(result, Err(PduError::UnsupportedVersion(0x02))));
    }

    #[test]
    fn test_tpkt_header_invalid_length() {
        let buffer = vec![0x03, 0x00, 0x00, 0x02]; // Too short length
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
        let test_cases = vec![vec![], vec![0x00], vec![0x01, 0x02, 0x03], vec![0xFF; 100]];

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
