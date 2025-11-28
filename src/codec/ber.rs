use crate::pdu::{PduError, Result};
use std::io::Write;

/// BER 태그 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BerTag {
    Boolean = 0x01,
    Integer = 0x02,
    OctetString = 0x04,
    Enumerated = 0x0A,
    Sequence = 0x30,
}

/// BER 클래스 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BerClass {
    Universal = 0x00,
    Application = 0x40,
    ContextSpecific = 0x80,
    Private = 0xC0,
}

/// BER Reader - BER 인코딩된 데이터를 읽기 위한 유틸리티
pub struct BerReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> BerReader<'a> {
    /// 새로운 BER Reader 생성
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    /// 현재 위치 반환
    pub fn position(&self) -> usize {
        self.position
    }

    /// 남은 바이트 수 반환
    pub fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.position)
    }

    /// BER 태그 읽기 (1바이트 태그만 지원)
    pub fn read_tag(&mut self) -> Result<u8> {
        if self.remaining() < 1 {
            return Err(PduError::InsufficientData {
                needed: 1,
                available: self.remaining(),
            });
        }

        let tag = self.buffer[self.position];
        self.position += 1;
        Ok(tag)
    }

    /// BER 길이 읽기 (short form, long form 지원)
    pub fn read_length(&mut self) -> Result<usize> {
        if self.remaining() < 1 {
            return Err(PduError::InsufficientData {
                needed: 1,
                available: self.remaining(),
            });
        }

        let first_byte = self.buffer[self.position];
        self.position += 1;

        // Short form: 최상위 비트가 0 (0-127)
        if first_byte & 0x80 == 0 {
            return Ok(first_byte as usize);
        }

        // Long form: 최상위 비트가 1
        let num_octets = (first_byte & 0x7F) as usize;

        if num_octets == 0 {
            // Indefinite form은 지원하지 않음
            return Err(PduError::ParseError(
                "Indefinite length not supported".to_string(),
            ));
        }

        if num_octets > 4 {
            return Err(PduError::ParseError(format!(
                "Length octets too large: {}",
                num_octets
            )));
        }

        if self.remaining() < num_octets {
            return Err(PduError::InsufficientData {
                needed: num_octets,
                available: self.remaining(),
            });
        }

        let mut length: usize = 0;
        for _ in 0..num_octets {
            length = (length << 8) | (self.buffer[self.position] as usize);
            self.position += 1;
        }

        Ok(length)
    }

    /// INTEGER 읽기 (BER 인코딩)
    pub fn read_integer(&mut self) -> Result<u32> {
        let tag = self.read_tag()?;
        if tag != BerTag::Integer as u8 {
            return Err(PduError::ParseError(format!(
                "Expected INTEGER tag (0x02), got 0x{:02x}",
                tag
            )));
        }

        let length = self.read_length()?;
        if length == 0 || length > 4 {
            return Err(PduError::ParseError(format!(
                "Invalid INTEGER length: {}",
                length
            )));
        }

        if self.remaining() < length {
            return Err(PduError::InsufficientData {
                needed: length,
                available: self.remaining(),
            });
        }

        let mut value: u32 = 0;
        for _ in 0..length {
            value = (value << 8) | (self.buffer[self.position] as u32);
            self.position += 1;
        }

        Ok(value)
    }

    /// OCTET STRING 읽기
    pub fn read_octet_string(&mut self) -> Result<Vec<u8>> {
        let tag = self.read_tag()?;
        if tag != BerTag::OctetString as u8 {
            return Err(PduError::ParseError(format!(
                "Expected OCTET STRING tag (0x04), got 0x{:02x}",
                tag
            )));
        }

        let length = self.read_length()?;
        self.read_bytes(length)
    }

    /// ENUMERATED 읽기
    pub fn read_enumerated(&mut self) -> Result<u8> {
        let tag = self.read_tag()?;
        if tag != BerTag::Enumerated as u8 {
            return Err(PduError::ParseError(format!(
                "Expected ENUMERATED tag (0x0A), got 0x{:02x}",
                tag
            )));
        }

        let length = self.read_length()?;
        if length != 1 {
            return Err(PduError::ParseError(format!(
                "Invalid ENUMERATED length: {}",
                length
            )));
        }

        if self.remaining() < 1 {
            return Err(PduError::InsufficientData {
                needed: 1,
                available: self.remaining(),
            });
        }

        let value = self.buffer[self.position];
        self.position += 1;
        Ok(value)
    }

    /// BOOLEAN 읽기
    pub fn read_boolean(&mut self) -> Result<bool> {
        let tag = self.read_tag()?;
        if tag != BerTag::Boolean as u8 {
            return Err(PduError::ParseError(format!(
                "Expected BOOLEAN tag (0x01), got 0x{:02x}",
                tag
            )));
        }

        let length = self.read_length()?;
        if length != 1 {
            return Err(PduError::ParseError(format!(
                "Invalid BOOLEAN length: {}",
                length
            )));
        }

        if self.remaining() < 1 {
            return Err(PduError::InsufficientData {
                needed: 1,
                available: self.remaining(),
            });
        }

        let value = self.buffer[self.position];
        self.position += 1;
        Ok(value != 0)
    }

    /// 지정된 길이만큼 바이트 읽기
    pub fn read_bytes(&mut self, length: usize) -> Result<Vec<u8>> {
        if self.remaining() < length {
            return Err(PduError::InsufficientData {
                needed: length,
                available: self.remaining(),
            });
        }

        let bytes = self.buffer[self.position..self.position + length].to_vec();
        self.position += length;
        Ok(bytes)
    }

    /// APPLICATION 태그가 있는 SEQUENCE 읽기
    pub fn read_application_tag(&mut self, expected_tag: u8) -> Result<usize> {
        let tag = self.read_tag()?;
        let expected = BerClass::Application as u8 | expected_tag;

        if tag != expected {
            return Err(PduError::ParseError(format!(
                "Expected APPLICATION tag 0x{:02x}, got 0x{:02x}",
                expected, tag
            )));
        }

        self.read_length()
    }

    /// Context-Specific 태그 읽기
    pub fn read_context_tag(&mut self, expected_tag: u8) -> Result<usize> {
        let tag = self.read_tag()?;
        let expected = BerClass::ContextSpecific as u8 | expected_tag;

        if tag != expected {
            return Err(PduError::ParseError(format!(
                "Expected CONTEXT tag 0x{:02x}, got 0x{:02x}",
                expected, tag
            )));
        }

        self.read_length()
    }
}

/// BER Writer - BER 인코딩 데이터를 쓰기 위한 유틸리티
pub struct BerWriter {
    buffer: Vec<u8>,
}

impl BerWriter {
    /// 새로운 BER Writer 생성
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// 버퍼를 소비하고 Vec<u8> 반환
    pub fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }

    /// 현재 버퍼 참조 반환
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// BER 태그 쓰기
    pub fn write_tag(&mut self, tag: u8) {
        self.buffer.push(tag);
    }

    /// BER 길이 쓰기 (short form, long form 자동 선택)
    pub fn write_length(&mut self, length: usize) {
        if length < 128 {
            // Short form
            self.buffer.push(length as u8);
        } else {
            // Long form
            let mut temp = length;
            let mut num_octets = 0;
            while temp > 0 {
                num_octets += 1;
                temp >>= 8;
            }

            // 첫 바이트: 0x80 | num_octets
            self.buffer.push(0x80 | num_octets);

            // 길이를 빅엔디안으로 쓰기
            for i in (0..num_octets).rev() {
                self.buffer.push(((length >> (i * 8)) & 0xFF) as u8);
            }
        }
    }

    /// INTEGER 쓰기
    pub fn write_integer(&mut self, value: u32) {
        self.write_tag(BerTag::Integer as u8);

        // 필요한 바이트 수 계산
        let bytes = if value == 0 {
            vec![0]
        } else {
            let mut temp = value;
            let mut bytes = Vec::new();
            while temp > 0 {
                bytes.push((temp & 0xFF) as u8);
                temp >>= 8;
            }
            bytes.reverse();

            // 최상위 비트가 1이면 0x00 패딩 추가 (음수로 해석되지 않도록)
            if bytes[0] & 0x80 != 0 {
                bytes.insert(0, 0x00);
            }

            bytes
        };

        self.write_length(bytes.len());
        self.buffer.extend_from_slice(&bytes);
    }

    /// OCTET STRING 쓰기
    pub fn write_octet_string(&mut self, data: &[u8]) {
        self.write_tag(BerTag::OctetString as u8);
        self.write_length(data.len());
        self.buffer.extend_from_slice(data);
    }

    /// BOOLEAN 쓰기
    pub fn write_boolean(&mut self, value: bool) {
        self.write_tag(BerTag::Boolean as u8);
        self.write_length(1);
        self.buffer.push(if value { 0xFF } else { 0x00 });
    }

    /// ENUMERATED 쓰기
    pub fn write_enumerated(&mut self, value: u8) {
        self.write_tag(BerTag::Enumerated as u8);
        self.write_length(1);
        self.buffer.push(value);
    }

    /// SEQUENCE 쓰기
    pub fn write_sequence<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let mut inner = BerWriter::new();
        f(&mut inner);

        self.write_tag(BerTag::Sequence as u8);
        self.write_length(inner.buffer.len());
        self.buffer.extend_from_slice(&inner.buffer);
    }

    /// APPLICATION 태그와 함께 쓰기
    pub fn write_application_tag<F>(&mut self, tag: u8, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let mut inner = BerWriter::new();
        f(&mut inner);

        self.write_tag(BerClass::Application as u8 | tag);
        self.write_length(inner.buffer.len());
        self.buffer.extend_from_slice(&inner.buffer);
    }

    /// Context-Specific 태그와 함께 쓰기
    pub fn write_context_tag<F>(&mut self, tag: u8, f: F)
    where
        F: FnOnce(&mut Self),
    {
        let mut inner = BerWriter::new();
        f(&mut inner);

        self.write_tag(BerClass::ContextSpecific as u8 | tag);
        self.write_length(inner.buffer.len());
        self.buffer.extend_from_slice(&inner.buffer);
    }
}

impl Default for BerWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for BerWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ber_length_short_form() {
        let mut writer = BerWriter::new();
        writer.write_length(127);
        assert_eq!(writer.as_bytes(), &[127]);

        let mut reader = BerReader::new(writer.as_bytes());
        assert_eq!(reader.read_length().unwrap(), 127);
    }

    #[test]
    fn test_ber_length_long_form() {
        let mut writer = BerWriter::new();
        writer.write_length(256);
        assert_eq!(writer.as_bytes(), &[0x82, 0x01, 0x00]);

        let mut reader = BerReader::new(writer.as_bytes());
        assert_eq!(reader.read_length().unwrap(), 256);
    }

    #[test]
    fn test_ber_integer() {
        let test_cases = vec![0, 1, 127, 128, 255, 256, 65535, 0xFFFFFF];

        for value in test_cases {
            let mut writer = BerWriter::new();
            writer.write_integer(value);

            let mut reader = BerReader::new(writer.as_bytes());
            assert_eq!(reader.read_integer().unwrap(), value);
        }
    }

    #[test]
    fn test_ber_octet_string() {
        let test_data = vec![0x01, 0x02, 0x03, 0xFF];

        let mut writer = BerWriter::new();
        writer.write_octet_string(&test_data);

        let mut reader = BerReader::new(writer.as_bytes());
        assert_eq!(reader.read_octet_string().unwrap(), test_data);
    }

    #[test]
    fn test_ber_enumerated() {
        let mut writer = BerWriter::new();
        writer.write_enumerated(5);

        let mut reader = BerReader::new(writer.as_bytes());
        assert_eq!(reader.read_enumerated().unwrap(), 5);
    }

    #[test]
    fn test_ber_sequence() {
        let mut writer = BerWriter::new();
        writer.write_sequence(|w| {
            w.write_integer(42);
            w.write_integer(100);
        });

        let mut reader = BerReader::new(writer.as_bytes());
        let tag = reader.read_tag().unwrap();
        assert_eq!(tag, BerTag::Sequence as u8);

        let length = reader.read_length().unwrap();
        assert!(length > 0);

        assert_eq!(reader.read_integer().unwrap(), 42);
        assert_eq!(reader.read_integer().unwrap(), 100);
    }

    #[test]
    fn test_ber_application_tag() {
        let mut writer = BerWriter::new();
        writer.write_application_tag(5, |w| {
            w.write_integer(123);
        });

        let mut reader = BerReader::new(writer.as_bytes());
        let length = reader.read_application_tag(5).unwrap();
        assert!(length > 0);

        assert_eq!(reader.read_integer().unwrap(), 123);
    }

    #[test]
    fn test_ber_context_tag() {
        let mut writer = BerWriter::new();
        writer.write_context_tag(3, |w| {
            w.write_integer(456);
        });

        let mut reader = BerReader::new(writer.as_bytes());
        let length = reader.read_context_tag(3).unwrap();
        assert!(length > 0);

        assert_eq!(reader.read_integer().unwrap(), 456);
    }

    #[test]
    fn test_ber_roundtrip_complex() {
        let mut writer = BerWriter::new();
        writer.write_sequence(|w| {
            w.write_integer(1);
            w.write_octet_string(b"test");
            w.write_enumerated(2);
            w.write_sequence(|w2| {
                w2.write_integer(99);
            });
        });

        let mut reader = BerReader::new(writer.as_bytes());
        reader.read_tag().unwrap(); // SEQUENCE tag
        reader.read_length().unwrap();

        assert_eq!(reader.read_integer().unwrap(), 1);
        assert_eq!(reader.read_octet_string().unwrap(), b"test");
        assert_eq!(reader.read_enumerated().unwrap(), 2);

        reader.read_tag().unwrap(); // nested SEQUENCE
        reader.read_length().unwrap();
        assert_eq!(reader.read_integer().unwrap(), 99);
    }
}
