use crate::pdu::{Pdu, PduError, PduWithHeader, Result};
use std::io::{Read, Write};

/// X.224 Data PDU Type code (base, without EOT)
pub const X224_DATA_TYPE: u8 = 0xF0;

/// EOT (End of Transmission) flag (least significant bit)
pub const EOT_FLAG: u8 = 0x01;

/// X.224 Data header minimum size
pub const X224_DATA_HEADER_MIN_SIZE: usize = 2;

/// X.224 Data header
///
/// ```text
/// +--------+--------+
/// |  LI    | Type   |
/// |        |+EOT    |
/// +--------+--------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataHeader {
    /// Length Indicator: X.224 header length - 1
    pub length_indicator: u8,
    /// PDU Type (0xF0 for Data)
    pub pdu_type: u8,
    /// End of Transmission flag
    pub eot: bool,
}

impl DataHeader {
    /// Create new X.224 Data header
    ///
    /// # Arguments
    /// * `eot` - End of Transmission flag
    pub fn new(eot: bool) -> Self {
        Self {
            // Base header size is 2 bytes, so LI is 2-1 = 1
            length_indicator: (X224_DATA_HEADER_MIN_SIZE - 1) as u8,
            pdu_type: X224_DATA_TYPE,
            eot,
        }
    }

    /// Encode header
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_all(&[self.length_indicator])?;

        let type_byte = if self.eot {
            self.pdu_type | EOT_FLAG
        } else {
            self.pdu_type
        };
        buffer.write_all(&[type_byte])?;

        Ok(())
    }

    /// Decode header
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let mut header_buf = [0u8; 2];
        buffer.read_exact(&mut header_buf)?;

        let length_indicator = header_buf[0];
        let type_byte = header_buf[1];

        // Extract EOT flag
        let eot = (type_byte & EOT_FLAG) != 0;

        // Verify PDU Type (excluding EOT flag)
        // Mask with 0xFE to remove least significant bit (EOT)
        let pdu_type = type_byte & 0xFE;
        if pdu_type != X224_DATA_TYPE {
            return Err(PduError::InvalidPduType(type_byte));
        }

        // Verify Length Indicator
        if length_indicator < (X224_DATA_HEADER_MIN_SIZE - 1) as u8 {
            return Err(PduError::InvalidLength {
                expected: X224_DATA_HEADER_MIN_SIZE - 1,
                actual: length_indicator as usize,
            });
        }

        // If LI is greater than minimum, skip extra bytes
        let extra_bytes = length_indicator as usize - (X224_DATA_HEADER_MIN_SIZE - 1);
        if extra_bytes > 0 {
            let mut skip_buf = vec![0u8; extra_bytes];
            buffer.read_exact(&mut skip_buf)?;
        }

        Ok(Self {
            length_indicator,
            pdu_type: X224_DATA_TYPE,
            eot,
        })
    }

    /// Return header size
    pub fn size(&self) -> usize {
        self.length_indicator as usize + 1
    }
}

/// X.224 Data PDU
///
/// X.224 Data Transfer PDU for transmitting RDP data
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPdu {
    /// X.224 Data header
    header: DataHeader,
    /// Payload (MCS data)
    payload: Vec<u8>,
}

impl DataPdu {
    /// Create new X.224 Data PDU
    ///
    /// # Arguments
    /// * `payload` - MCS payload
    /// * `eot` - End of Transmission flag (default: true)
    pub fn new(payload: Vec<u8>) -> Self {
        Self::new_with_eot(payload, true)
    }

    /// Create new X.224 Data PDU with specified EOT flag
    pub fn new_with_eot(payload: Vec<u8>, eot: bool) -> Self {
        let header = DataHeader::new(eot);
        Self { header, payload }
    }

    /// Return payload reference
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Transfer payload ownership
    pub fn into_payload(self) -> Vec<u8> {
        self.payload
    }

    /// Return mutable payload reference
    pub fn payload_mut(&mut self) -> &mut Vec<u8> {
        &mut self.payload
    }

    /// Return EOT flag
    pub fn eot(&self) -> bool {
        self.header.eot
    }
}

impl Pdu for DataPdu {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        self.header.encode(buffer)?;
        buffer.write_all(&self.payload)?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let header = DataHeader::decode(buffer)?;

        // Read all remaining bytes as payload
        let mut payload = Vec::new();
        buffer.read_to_end(&mut payload)?;

        Ok(Self { header, payload })
    }

    fn size(&self) -> usize {
        self.header.size() + self.payload.len()
    }
}

impl PduWithHeader for DataPdu {
    type Header = DataHeader;

    fn header(&self) -> &Self::Header {
        &self.header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdu::x224::PduType;
    use std::io::Cursor;

    #[test]
    fn test_data_header_new() {
        let header = DataHeader::new(true);
        assert_eq!(header.length_indicator, 1);
        assert_eq!(header.pdu_type, X224_DATA_TYPE);
        assert_eq!(header.eot, true);

        let header = DataHeader::new(false);
        assert_eq!(header.eot, false);
    }

    #[test]
    fn test_data_header_encode_decode_with_eot() {
        let original = DataHeader::new(true);
        let mut buffer = Vec::new();
        original.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0], 1); // LI
        assert_eq!(buffer[1], 0xF1); // 0xF0 | 0x01

        let mut cursor = Cursor::new(buffer);
        let decoded = DataHeader::decode(&mut cursor).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_data_header_encode_decode_without_eot() {
        let original = DataHeader::new(false);
        let mut buffer = Vec::new();
        original.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer[0], 1); // LI
        assert_eq!(buffer[1], 0xF0); // Data without EOT

        let mut cursor = Cursor::new(buffer);
        let decoded = DataHeader::decode(&mut cursor).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_data_header_invalid_type() {
        let buffer = vec![1, 0xE0]; // Invalid PDU Type (Connection Request)
        let mut cursor = Cursor::new(buffer);
        let result = DataHeader::decode(&mut cursor);

        assert!(matches!(result, Err(PduError::InvalidPduType(0xE0))));
    }

    #[test]
    fn test_data_pdu_new() {
        let payload = vec![1, 2, 3, 4, 5];
        let pdu = DataPdu::new(payload.clone());

        assert_eq!(pdu.payload(), &payload[..]);
        assert_eq!(pdu.eot(), true);
        assert_eq!(pdu.size(), 7); // 2 (header) + 5 (payload)
    }

    #[test]
    fn test_data_pdu_encode_decode() {
        let payload = vec![0x11, 0x22, 0x33, 0x44];
        let original = DataPdu::new(payload);

        let mut buffer = Vec::new();
        original.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), 6); // 2 (header) + 4 (payload)

        let mut cursor = Cursor::new(buffer);
        let decoded = DataPdu::decode(&mut cursor).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_data_pdu_roundtrip() {
        let test_cases = vec![
            (vec![], true),
            (vec![0x00], false),
            (vec![0x01, 0x02, 0x03], true),
            (vec![0xFF; 100], false),
        ];

        for (payload, eot) in test_cases {
            let original = DataPdu::new_with_eot(payload, eot);
            let mut buffer = Vec::new();
            original.encode(&mut buffer).unwrap();

            let mut cursor = Cursor::new(buffer);
            let decoded = DataPdu::decode(&mut cursor).unwrap();

            assert_eq!(original, decoded);
            assert_eq!(decoded.eot(), eot);
        }
    }

    #[test]
    fn test_pdu_type_from_u8() {
        assert_eq!(PduType::from_u8(0xE0), Some(PduType::ConnectionRequest));
        assert_eq!(PduType::from_u8(0xD0), Some(PduType::ConnectionConfirm));
        assert_eq!(PduType::from_u8(0x80), Some(PduType::DisconnectRequest));
        assert_eq!(PduType::from_u8(0xF0), Some(PduType::Data));
        assert_eq!(PduType::from_u8(0xF1), Some(PduType::Data)); // Includes EOT flag
        assert_eq!(PduType::from_u8(0x70), Some(PduType::Error));
        assert_eq!(PduType::from_u8(0x99), None);
    }
}
