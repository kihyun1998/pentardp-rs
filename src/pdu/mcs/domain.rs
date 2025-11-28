use crate::codec::{BerReader, BerWriter};
use crate::pdu::{Pdu, PduError, Result};
use std::io::{Read, Write};

/// MCS Erect Domain Request TAG ([APPLICATION 1])
pub const MCS_ERECT_DOMAIN_REQUEST: u8 = 1;

/// MCS Attach User Request TAG ([APPLICATION 10])
pub const MCS_ATTACH_USER_REQUEST: u8 = 10;

/// MCS Attach User Confirm TAG ([APPLICATION 11])
pub const MCS_ATTACH_USER_CONFIRM: u8 = 11;

/// MCS Result codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum McsResult {
    RtSuccessful = 0,
    RtDomainMerging = 1,
    RtDomainNotHierarchical = 2,
    RtNoSuchChannel = 3,
    RtNoSuchDomain = 4,
    RtNoSuchUser = 5,
    RtNotAdmitted = 6,
    RtOtherUserIdInvalid = 7,
    RtParametersUnacceptable = 8,
    RtTokenNotAvailable = 9,
    RtTokenNotPossessed = 10,
    RtTooManyChannels = 11,
    RtTooManyTokens = 12,
    RtTooManyUsers = 13,
    RtUnspecifiedFailure = 14,
    RtUserRejected = 15,
}

impl McsResult {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(McsResult::RtSuccessful),
            1 => Some(McsResult::RtDomainMerging),
            2 => Some(McsResult::RtDomainNotHierarchical),
            3 => Some(McsResult::RtNoSuchChannel),
            4 => Some(McsResult::RtNoSuchDomain),
            5 => Some(McsResult::RtNoSuchUser),
            6 => Some(McsResult::RtNotAdmitted),
            7 => Some(McsResult::RtOtherUserIdInvalid),
            8 => Some(McsResult::RtParametersUnacceptable),
            9 => Some(McsResult::RtTokenNotAvailable),
            10 => Some(McsResult::RtTokenNotPossessed),
            11 => Some(McsResult::RtTooManyChannels),
            12 => Some(McsResult::RtTooManyTokens),
            13 => Some(McsResult::RtTooManyUsers),
            14 => Some(McsResult::RtUnspecifiedFailure),
            15 => Some(McsResult::RtUserRejected),
            _ => None,
        }
    }
}

/// MCS Erect Domain Request
///
/// ErectDomainRequest ::= [APPLICATION 1] IMPLICIT SEQUENCE {
///     subHeight   Integer,
///     subInterval Integer
/// }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErectDomainRequest {
    /// Sub-height (usually 0)
    pub sub_height: u32,
    /// Sub-interval (usually 0)
    pub sub_interval: u32,
}

impl ErectDomainRequest {
    /// Create new ErectDomainRequest
    pub fn new(sub_height: u32, sub_interval: u32) -> Self {
        Self {
            sub_height,
            sub_interval,
        }
    }

    /// Create with default values (sub_height=0, sub_interval=0)
    pub fn default_request() -> Self {
        Self::new(0, 0)
    }
}

impl Pdu for ErectDomainRequest {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let mut writer = BerWriter::new();

        writer.write_application_tag(MCS_ERECT_DOMAIN_REQUEST, |w| {
            w.write_integer(self.sub_height);
            w.write_integer(self.sub_interval);
        });

        buffer.write_all(writer.as_bytes())?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;

        let mut reader = BerReader::new(&data);
        reader.read_application_tag(MCS_ERECT_DOMAIN_REQUEST)?;

        let sub_height = reader.read_integer()?;
        let sub_interval = reader.read_integer()?;

        Ok(Self {
            sub_height,
            sub_interval,
        })
    }

    fn size(&self) -> usize {
        let mut writer = BerWriter::new();
        self.encode(&mut writer).unwrap();
        writer.as_bytes().len()
    }
}

/// MCS Attach User Request
///
/// AttachUserRequest ::= [APPLICATION 10] IMPLICIT SEQUENCE {}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachUserRequest;

impl AttachUserRequest {
    /// Create new AttachUserRequest
    pub fn new() -> Self {
        Self
    }
}

impl Default for AttachUserRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl Pdu for AttachUserRequest {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let mut writer = BerWriter::new();

        writer.write_application_tag(MCS_ATTACH_USER_REQUEST, |_w| {
            // Empty sequence
        });

        buffer.write_all(writer.as_bytes())?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;

        let mut reader = BerReader::new(&data);
        reader.read_application_tag(MCS_ATTACH_USER_REQUEST)?;

        Ok(Self)
    }

    fn size(&self) -> usize {
        let mut writer = BerWriter::new();
        self.encode(&mut writer).unwrap();
        writer.as_bytes().len()
    }
}

/// MCS Attach User Confirm
///
/// AttachUserConfirm ::= [APPLICATION 11] IMPLICIT SEQUENCE {
///     result     Result,
///     initiator  UserId OPTIONAL
/// }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachUserConfirm {
    /// Result code
    pub result: McsResult,
    /// User ID (initiator)
    pub user_id: Option<u16>,
}

impl AttachUserConfirm {
    /// Create new AttachUserConfirm
    pub fn new(result: McsResult, user_id: Option<u16>) -> Self {
        Self { result, user_id }
    }

    /// Create success response
    pub fn success(user_id: u16) -> Self {
        Self::new(McsResult::RtSuccessful, Some(user_id))
    }

    /// Create failure response
    pub fn failure(result: McsResult) -> Self {
        Self::new(result, None)
    }
}

impl Pdu for AttachUserConfirm {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let mut writer = BerWriter::new();

        writer.write_application_tag(MCS_ATTACH_USER_CONFIRM, |w| {
            w.write_enumerated(self.result as u8);
            if let Some(user_id) = self.user_id {
                w.write_integer(user_id as u32);
            }
        });

        buffer.write_all(writer.as_bytes())?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;

        let mut reader = BerReader::new(&data);
        reader.read_application_tag(MCS_ATTACH_USER_CONFIRM)?;

        let result_code = reader.read_enumerated()?;
        let result = McsResult::from_u8(result_code).ok_or_else(|| {
            PduError::ParseError(format!("Invalid MCS result code: {}", result_code))
        })?;

        let user_id = if reader.remaining() > 0 {
            Some(reader.read_integer()? as u16)
        } else {
            None
        };

        Ok(Self { result, user_id })
    }

    fn size(&self) -> usize {
        let mut writer = BerWriter::new();
        self.encode(&mut writer).unwrap();
        writer.as_bytes().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_erect_domain_request() {
        let request = ErectDomainRequest::new(0, 0);

        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ErectDomainRequest::decode(&mut cursor).unwrap();

        assert_eq!(request, decoded);
        assert_eq!(decoded.sub_height, 0);
        assert_eq!(decoded.sub_interval, 0);
    }

    #[test]
    fn test_erect_domain_request_default() {
        let request = ErectDomainRequest::default_request();
        assert_eq!(request.sub_height, 0);
        assert_eq!(request.sub_interval, 0);
    }

    #[test]
    fn test_erect_domain_request_with_values() {
        let request = ErectDomainRequest::new(5, 10);

        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ErectDomainRequest::decode(&mut cursor).unwrap();

        assert_eq!(request, decoded);
        assert_eq!(decoded.sub_height, 5);
        assert_eq!(decoded.sub_interval, 10);
    }

    #[test]
    fn test_attach_user_request() {
        let request = AttachUserRequest::new();

        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        assert!(buffer.len() > 0);

        let mut cursor = Cursor::new(buffer);
        let decoded = AttachUserRequest::decode(&mut cursor).unwrap();

        assert_eq!(request, decoded);
    }

    #[test]
    fn test_attach_user_confirm_success() {
        let confirm = AttachUserConfirm::success(1001);

        let mut buffer = Vec::new();
        confirm.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = AttachUserConfirm::decode(&mut cursor).unwrap();

        assert_eq!(confirm, decoded);
        assert_eq!(decoded.result, McsResult::RtSuccessful);
        assert_eq!(decoded.user_id, Some(1001));
    }

    #[test]
    fn test_attach_user_confirm_failure() {
        let confirm = AttachUserConfirm::failure(McsResult::RtTooManyUsers);

        let mut buffer = Vec::new();
        confirm.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = AttachUserConfirm::decode(&mut cursor).unwrap();

        assert_eq!(confirm, decoded);
        assert_eq!(decoded.result, McsResult::RtTooManyUsers);
        assert_eq!(decoded.user_id, None);
    }

    #[test]
    fn test_attach_user_confirm_roundtrip() {
        let test_cases = vec![
            AttachUserConfirm::success(1),
            AttachUserConfirm::success(65535),
            AttachUserConfirm::failure(McsResult::RtNoSuchUser),
            AttachUserConfirm::failure(McsResult::RtUserRejected),
        ];

        for confirm in test_cases {
            let mut buffer = Vec::new();
            confirm.encode(&mut buffer).unwrap();

            let mut cursor = Cursor::new(buffer);
            let decoded = AttachUserConfirm::decode(&mut cursor).unwrap();

            assert_eq!(confirm, decoded);
        }
    }
}
