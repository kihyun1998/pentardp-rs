use crate::pdu::{Pdu, PduError, PduWithHeader, Result};
use std::io::{Read, Write};

/// X.224 Data PDU 타입 코드 (base, without EOT)
pub const X224_DATA_TYPE: u8 = 0xF0;

/// EOT (End of Transmission) 플래그 (최하위 비트)
pub const EOT_FLAG: u8 = 0x01;

/// X.224 Data 헤더 최소 크기
pub const X224_DATA_HEADER_MIN_SIZE: usize = 2;

/// X.224 Data 헤더
///
/// ```text
/// +--------+--------+
/// |  LI    | Type   |
/// |        |+EOT    |
/// +--------+--------+
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataHeader {
    /// Length Indicator: X.224 헤더 길이 - 1
    pub length_indicator: u8,
    /// PDU Type (0xF0 for Data)
    pub pdu_type: u8,
    /// End of Transmission 플래그
    pub eot: bool,
}

impl DataHeader {
    /// 새로운 X.224 Data 헤더 생성
    ///
    /// # Arguments
    /// * `eot` - End of Transmission 플래그
    pub fn new(eot: bool) -> Self {
        Self {
            // 기본 헤더 크기는 2바이트이므로 LI는 2-1 = 1
            length_indicator: (X224_DATA_HEADER_MIN_SIZE - 1) as u8,
            pdu_type: X224_DATA_TYPE,
            eot,
        }
    }

    /// 헤더 인코딩
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

    /// 헤더 디코딩
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let mut header_buf = [0u8; 2];
        buffer.read_exact(&mut header_buf)?;

        let length_indicator = header_buf[0];
        let type_byte = header_buf[1];

        // EOT 플래그 추출
        let eot = (type_byte & EOT_FLAG) != 0;

        // PDU 타입 검증 (EOT 플래그 제외)
        // 0xFE로 마스킹하여 최하위 비트(EOT) 제거
        let pdu_type = type_byte & 0xFE;
        if pdu_type != X224_DATA_TYPE {
            return Err(PduError::InvalidPduType(type_byte));
        }

        // Length Indicator 검증
        if length_indicator < (X224_DATA_HEADER_MIN_SIZE - 1) as u8 {
            return Err(PduError::InvalidLength {
                expected: X224_DATA_HEADER_MIN_SIZE - 1,
                actual: length_indicator as usize,
            });
        }

        // LI가 최소값보다 크면 추가 바이트를 건너뛰어야 함
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

    /// 헤더 크기 반환
    pub fn size(&self) -> usize {
        self.length_indicator as usize + 1
    }
}

/// X.224 Data PDU
///
/// RDP 데이터를 전송하는 X.224 Data Transfer PDU
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPdu {
    /// X.224 Data 헤더
    header: DataHeader,
    /// 페이로드 (MCS 데이터)
    payload: Vec<u8>,
}

impl DataPdu {
    /// 새로운 X.224 Data PDU 생성
    ///
    /// # Arguments
    /// * `payload` - MCS 페이로드
    /// * `eot` - End of Transmission 플래그 (기본값: true)
    pub fn new(payload: Vec<u8>) -> Self {
        Self::new_with_eot(payload, true)
    }

    /// EOT 플래그를 지정하여 새로운 X.224 Data PDU 생성
    pub fn new_with_eot(payload: Vec<u8>, eot: bool) -> Self {
        let header = DataHeader::new(eot);
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

    /// EOT 플래그 반환
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

        // 나머지 모든 바이트를 페이로드로 읽음
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
        let buffer = vec![1, 0xE0]; // 잘못된 PDU 타입 (Connection Request)
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
        assert_eq!(PduType::from_u8(0xF1), Some(PduType::Data)); // EOT 플래그 포함
        assert_eq!(PduType::from_u8(0x70), Some(PduType::Error));
        assert_eq!(PduType::from_u8(0x99), None);
    }
}
