use crate::pdu::{Pdu, PduError, PduWithHeader, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// X.224 Connection Request PDU Type
pub const X224_CR_TYPE: u8 = 0xE0;

/// X.224 Connection Confirm PDU Type
pub const X224_CC_TYPE: u8 = 0xD0;

/// X.224 Connection header minimum size (LI + Type + DST-REF + SRC-REF + Class)
pub const X224_CONNECTION_HEADER_MIN_SIZE: usize = 7;

/// RDP Negotiation Request Type
pub const RDP_NEG_REQ: u8 = 0x01;

/// RDP Negotiation Response Type
pub const RDP_NEG_RSP: u8 = 0x02;

/// RDP Negotiation Failure Type
pub const RDP_NEG_FAILURE: u8 = 0x03;

/// RDP Negotiation structure size
pub const RDP_NEG_DATA_SIZE: usize = 8;

/// RDP protocol flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// Standard RDP Security (0x00)
    RdpSecurity = 0x00,
    /// TLS 1.0 Security (0x01)
    Ssl = 0x01,
    /// CredSSP Security (0x02)
    Hybrid = 0x02,
    /// RDSTLS Security (0x04)
    RdsTls = 0x04,
    /// Hybrid Extended Security (0x08)
    HybridEx = 0x08,
}

impl Protocol {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x00 => Some(Protocol::RdpSecurity),
            0x01 => Some(Protocol::Ssl),
            0x02 => Some(Protocol::Hybrid),
            0x04 => Some(Protocol::RdsTls),
            0x08 => Some(Protocol::HybridEx),
            _ => None,
        }
    }
}

/// RDP Negotiation Request/Response
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RdpNegotiation {
    /// Type (REQ=0x01, RSP=0x02, FAILURE=0x03)
    pub neg_type: u8,
    /// Flags
    pub flags: u8,
    /// Selected Protocol
    pub selected_protocol: u32,
}

impl RdpNegotiation {
    /// Create RDP Negotiation Request
    pub fn new_request(protocol: u32) -> Self {
        Self {
            neg_type: RDP_NEG_REQ,
            flags: 0,
            selected_protocol: protocol,
        }
    }

    /// Create RDP Negotiation Response
    pub fn new_response(protocol: u32) -> Self {
        Self {
            neg_type: RDP_NEG_RSP,
            flags: 0,
            selected_protocol: protocol,
        }
    }

    /// Encode
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u8(self.neg_type)?;
        buffer.write_u8(self.flags)?;
        buffer.write_u16::<LittleEndian>(RDP_NEG_DATA_SIZE as u16)?;
        buffer.write_u32::<LittleEndian>(self.selected_protocol)?;
        Ok(())
    }

    /// Decode
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let neg_type = buffer.read_u8()?;
        let flags = buffer.read_u8()?;
        let length = buffer.read_u16::<LittleEndian>()?;

        if length != RDP_NEG_DATA_SIZE as u16 {
            return Err(PduError::InvalidLength {
                expected: RDP_NEG_DATA_SIZE,
                actual: length as usize,
            });
        }

        let selected_protocol = buffer.read_u32::<LittleEndian>()?;

        Ok(Self {
            neg_type,
            flags,
            selected_protocol,
        })
    }

    /// Return size
    pub fn size(&self) -> usize {
        RDP_NEG_DATA_SIZE
    }
}

/// X.224 Connection header
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionHeader {
    /// Length Indicator: X.224 header length - 1
    pub length_indicator: u8,
    /// PDU Type (0xE0 for CR, 0xD0 for CC)
    pub pdu_type: u8,
    /// Destination Reference
    pub dst_ref: u16,
    /// Source Reference
    pub src_ref: u16,
    /// Class and Option
    pub class_option: u8,
}

impl ConnectionHeader {
    /// Create new Connection Request header
    pub fn new_request(src_ref: u16) -> Self {
        Self {
            length_indicator: 6, // 7 - 1
            pdu_type: X224_CR_TYPE,
            dst_ref: 0,
            src_ref,
            class_option: 0,
        }
    }

    /// Create new Connection Confirm header
    pub fn new_confirm(dst_ref: u16, src_ref: u16) -> Self {
        Self {
            length_indicator: 6, // 7 - 1
            pdu_type: X224_CC_TYPE,
            dst_ref,
            src_ref,
            class_option: 0,
        }
    }

    /// Encode header
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u8(self.length_indicator)?;
        buffer.write_u8(self.pdu_type)?;
        buffer.write_u16::<LittleEndian>(self.dst_ref)?;
        buffer.write_u16::<LittleEndian>(self.src_ref)?;
        buffer.write_u8(self.class_option)?;
        Ok(())
    }

    /// Decode header (does not skip variable part)
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let length_indicator = buffer.read_u8()?;
        let pdu_type = buffer.read_u8()?;

        // Verify PDU Type
        if pdu_type != X224_CR_TYPE && pdu_type != X224_CC_TYPE {
            return Err(PduError::InvalidPduType(pdu_type));
        }

        let dst_ref = buffer.read_u16::<LittleEndian>()?;
        let src_ref = buffer.read_u16::<LittleEndian>()?;
        let class_option = buffer.read_u8()?;

        // Variable part is not skipped here
        // Processed in each PDU (ConnectionRequest/ConnectionConfirm)

        Ok(Self {
            length_indicator,
            pdu_type,
            dst_ref,
            src_ref,
            class_option,
        })
    }

    /// Return header size
    pub fn size(&self) -> usize {
        self.length_indicator as usize + 1
    }
}

/// X.224 Connection Request PDU
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionRequest {
    /// Connection header
    header: ConnectionHeader,
    /// Cookie (optional)
    cookie: Option<String>,
    /// RDP Negotiation Request (optional)
    rdp_negotiation: Option<RdpNegotiation>,
}

impl ConnectionRequest {
    /// Create new Connection Request
    pub fn new(src_ref: u16) -> Self {
        Self {
            header: ConnectionHeader::new_request(src_ref),
            cookie: None,
            rdp_negotiation: None,
        }
    }

    /// Set Cookie
    pub fn with_cookie(mut self, username: &str) -> Self {
        self.cookie = Some(format!("Cookie: mstshash={}\r\n", username));
        self.update_length_indicator();
        self
    }

    /// Set RDP Negotiation Request
    pub fn with_negotiation(mut self, protocol: u32) -> Self {
        self.rdp_negotiation = Some(RdpNegotiation::new_request(protocol));
        self.update_length_indicator();
        self
    }

    /// Update Length Indicator
    fn update_length_indicator(&mut self) {
        let variable_size = self.cookie.as_ref().map(|c| c.len()).unwrap_or(0)
            + self.rdp_negotiation.as_ref().map(|n| n.size()).unwrap_or(0);

        self.header.length_indicator = (X224_CONNECTION_HEADER_MIN_SIZE - 1 + variable_size) as u8;
    }

    /// Return Cookie
    pub fn cookie(&self) -> Option<&str> {
        self.cookie.as_deref()
    }

    /// Return RDP Negotiation
    pub fn rdp_negotiation(&self) -> Option<&RdpNegotiation> {
        self.rdp_negotiation.as_ref()
    }
}

impl Pdu for ConnectionRequest {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        self.header.encode(buffer)?;

        if let Some(ref cookie) = self.cookie {
            buffer.write_all(cookie.as_bytes())?;
        }

        if let Some(ref negotiation) = self.rdp_negotiation {
            negotiation.encode(buffer)?;
        }

        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let header = ConnectionHeader::decode(buffer)?;

        if header.pdu_type != X224_CR_TYPE {
            return Err(PduError::InvalidPduType(header.pdu_type));
        }

        // Read variable part
        let variable_length =
            (header.length_indicator as usize + 1).saturating_sub(X224_CONNECTION_HEADER_MIN_SIZE);

        let mut cookie = None;
        let mut rdp_negotiation = None;

        if variable_length > 0 {
            let mut variable_data = vec![0u8; variable_length];
            buffer.read_exact(&mut variable_data)?;

            let mut cursor = std::io::Cursor::new(&variable_data);

            // Parse Cookie
            if let Ok(cookie_str) = std::str::from_utf8(&variable_data) {
                if cookie_str.starts_with("Cookie: mstshash=") {
                    if let Some(end) = cookie_str.find("\r\n") {
                        cookie = Some(cookie_str[..end + 2].to_string());
                        cursor.set_position((end + 2) as u64);
                    }
                }
            }

            // Parse RDP Negotiation
            if cursor.position() < variable_data.len() as u64 {
                if let Ok(negotiation) = RdpNegotiation::decode(&mut cursor) {
                    rdp_negotiation = Some(negotiation);
                }
            }
        }

        Ok(Self {
            header,
            cookie,
            rdp_negotiation,
        })
    }

    fn size(&self) -> usize {
        self.header.size()
            + self.cookie.as_ref().map(|c| c.len()).unwrap_or(0)
            + self.rdp_negotiation.as_ref().map(|n| n.size()).unwrap_or(0)
    }
}

impl PduWithHeader for ConnectionRequest {
    type Header = ConnectionHeader;

    fn header(&self) -> &Self::Header {
        &self.header
    }
}

/// X.224 Connection Confirm PDU
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionConfirm {
    /// Connection header
    header: ConnectionHeader,
    /// RDP Negotiation Response (optional)
    rdp_negotiation: Option<RdpNegotiation>,
}

impl ConnectionConfirm {
    /// Create new Connection Confirm
    pub fn new(dst_ref: u16, src_ref: u16) -> Self {
        Self {
            header: ConnectionHeader::new_confirm(dst_ref, src_ref),
            rdp_negotiation: None,
        }
    }

    /// Set RDP Negotiation Response
    pub fn with_negotiation(mut self, protocol: u32) -> Self {
        self.rdp_negotiation = Some(RdpNegotiation::new_response(protocol));
        self.header.length_indicator =
            (X224_CONNECTION_HEADER_MIN_SIZE - 1 + RDP_NEG_DATA_SIZE) as u8;
        self
    }

    /// Return RDP Negotiation
    pub fn rdp_negotiation(&self) -> Option<&RdpNegotiation> {
        self.rdp_negotiation.as_ref()
    }
}

impl Pdu for ConnectionConfirm {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        self.header.encode(buffer)?;

        if let Some(ref negotiation) = self.rdp_negotiation {
            negotiation.encode(buffer)?;
        }

        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let header = ConnectionHeader::decode(buffer)?;

        if header.pdu_type != X224_CC_TYPE {
            return Err(PduError::InvalidPduType(header.pdu_type));
        }

        // Read variable part
        let variable_length =
            (header.length_indicator as usize + 1).saturating_sub(X224_CONNECTION_HEADER_MIN_SIZE);

        let mut rdp_negotiation = None;

        if variable_length > 0 {
            let mut variable_data = vec![0u8; variable_length];
            buffer.read_exact(&mut variable_data)?;

            let mut cursor = std::io::Cursor::new(&variable_data);

            // Parse RDP Negotiation
            if let Ok(negotiation) = RdpNegotiation::decode(&mut cursor) {
                rdp_negotiation = Some(negotiation);
            }
        }

        Ok(Self {
            header,
            rdp_negotiation,
        })
    }

    fn size(&self) -> usize {
        self.header.size() + self.rdp_negotiation.as_ref().map(|n| n.size()).unwrap_or(0)
    }
}

impl PduWithHeader for ConnectionConfirm {
    type Header = ConnectionHeader;

    fn header(&self) -> &Self::Header {
        &self.header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_rdp_negotiation_request() {
        let negotiation = RdpNegotiation::new_request(Protocol::Ssl as u32);
        let mut buffer = Vec::new();
        negotiation.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), RDP_NEG_DATA_SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = RdpNegotiation::decode(&mut cursor).unwrap();

        assert_eq!(negotiation, decoded);
        assert_eq!(decoded.neg_type, RDP_NEG_REQ);
        assert_eq!(decoded.selected_protocol, Protocol::Ssl as u32);
    }

    #[test]
    fn test_rdp_negotiation_response() {
        let negotiation = RdpNegotiation::new_response(Protocol::Hybrid as u32);
        let mut buffer = Vec::new();
        negotiation.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = RdpNegotiation::decode(&mut cursor).unwrap();

        assert_eq!(negotiation, decoded);
        assert_eq!(decoded.neg_type, RDP_NEG_RSP);
    }

    #[test]
    fn test_connection_request_basic() {
        let request = ConnectionRequest::new(0x1234);

        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), X224_CONNECTION_HEADER_MIN_SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectionRequest::decode(&mut cursor).unwrap();

        assert_eq!(request, decoded);
        assert_eq!(decoded.header().src_ref, 0x1234);
        assert_eq!(decoded.header().dst_ref, 0);
    }

    #[test]
    fn test_connection_request_with_cookie() {
        let request = ConnectionRequest::new(0x1234).with_cookie("testuser");

        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectionRequest::decode(&mut cursor).unwrap();

        assert_eq!(decoded.cookie(), Some("Cookie: mstshash=testuser\r\n"));
    }

    #[test]
    fn test_connection_request_with_negotiation() {
        let request = ConnectionRequest::new(0x1234).with_negotiation(Protocol::Ssl as u32);

        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectionRequest::decode(&mut cursor).unwrap();

        assert!(decoded.rdp_negotiation().is_some());
        assert_eq!(
            decoded.rdp_negotiation().unwrap().selected_protocol,
            Protocol::Ssl as u32
        );
    }

    #[test]
    fn test_connection_request_with_cookie_and_negotiation() {
        let request = ConnectionRequest::new(0x1234)
            .with_cookie("admin")
            .with_negotiation(Protocol::Hybrid as u32);

        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectionRequest::decode(&mut cursor).unwrap();

        assert_eq!(decoded.cookie(), Some("Cookie: mstshash=admin\r\n"));
        assert!(decoded.rdp_negotiation().is_some());
        assert_eq!(
            decoded.rdp_negotiation().unwrap().selected_protocol,
            Protocol::Hybrid as u32
        );
    }

    #[test]
    fn test_connection_confirm_basic() {
        let confirm = ConnectionConfirm::new(0x1234, 0x5678);

        let mut buffer = Vec::new();
        confirm.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), X224_CONNECTION_HEADER_MIN_SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectionConfirm::decode(&mut cursor).unwrap();

        assert_eq!(confirm, decoded);
        assert_eq!(decoded.header().dst_ref, 0x1234);
        assert_eq!(decoded.header().src_ref, 0x5678);
    }

    #[test]
    fn test_connection_confirm_with_negotiation() {
        let confirm = ConnectionConfirm::new(0x1234, 0x5678).with_negotiation(Protocol::Ssl as u32);

        let mut buffer = Vec::new();
        confirm.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectionConfirm::decode(&mut cursor).unwrap();

        assert!(decoded.rdp_negotiation().is_some());
        assert_eq!(
            decoded.rdp_negotiation().unwrap().selected_protocol,
            Protocol::Ssl as u32
        );
    }

    #[test]
    fn test_connection_roundtrip() {
        let request = ConnectionRequest::new(0xABCD)
            .with_cookie("testuser123")
            .with_negotiation(Protocol::HybridEx as u32);

        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectionRequest::decode(&mut cursor).unwrap();

        assert_eq!(request, decoded);
    }
}
