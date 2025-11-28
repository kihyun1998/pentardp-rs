use crate::codec::{BerReader, BerWriter};
use crate::pdu::{Pdu, PduError, Result};
use std::io::{Read, Write};

use super::domain::McsResult;

/// MCS Channel Join Request TAG ([APPLICATION 14])
pub const MCS_CHANNEL_JOIN_REQUEST: u8 = 14;

/// MCS Channel Join Confirm TAG ([APPLICATION 15])
pub const MCS_CHANNEL_JOIN_CONFIRM: u8 = 15;

/// MCS Channel Join Request
///
/// ChannelJoinRequest ::= [APPLICATION 14] IMPLICIT SEQUENCE {
///     initiator  UserId,
///     channelId  ChannelId
/// }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelJoinRequest {
    /// User ID (initiator)
    pub user_id: u16,
    /// Channel ID to join
    pub channel_id: u16,
}

impl ChannelJoinRequest {
    /// 새로운 ChannelJoinRequest 생성
    pub fn new(user_id: u16, channel_id: u16) -> Self {
        Self {
            user_id,
            channel_id,
        }
    }
}

impl Pdu for ChannelJoinRequest {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let mut writer = BerWriter::new();

        writer.write_application_tag(MCS_CHANNEL_JOIN_REQUEST, |w| {
            w.write_integer(self.user_id as u32);
            w.write_integer(self.channel_id as u32);
        });

        buffer.write_all(writer.as_bytes())?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;

        let mut reader = BerReader::new(&data);
        reader.read_application_tag(MCS_CHANNEL_JOIN_REQUEST)?;

        let user_id = reader.read_integer()? as u16;
        let channel_id = reader.read_integer()? as u16;

        Ok(Self {
            user_id,
            channel_id,
        })
    }

    fn size(&self) -> usize {
        let mut writer = BerWriter::new();
        self.encode(&mut writer).unwrap();
        writer.as_bytes().len()
    }
}

/// MCS Channel Join Confirm
///
/// ChannelJoinConfirm ::= [APPLICATION 15] IMPLICIT SEQUENCE {
///     result      Result,
///     initiator   UserId,
///     requested   ChannelId,
///     channelId   ChannelId OPTIONAL
/// }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelJoinConfirm {
    /// Result code
    pub result: McsResult,
    /// User ID (initiator)
    pub user_id: u16,
    /// Requested Channel ID
    pub requested_channel_id: u16,
    /// Actual Channel ID (usually same as requested)
    pub channel_id: Option<u16>,
}

impl ChannelJoinConfirm {
    /// 새로운 ChannelJoinConfirm 생성
    pub fn new(
        result: McsResult,
        user_id: u16,
        requested_channel_id: u16,
        channel_id: Option<u16>,
    ) -> Self {
        Self {
            result,
            user_id,
            requested_channel_id,
            channel_id,
        }
    }

    /// 성공 응답 생성
    pub fn success(user_id: u16, channel_id: u16) -> Self {
        Self::new(
            McsResult::RtSuccessful,
            user_id,
            channel_id,
            Some(channel_id),
        )
    }

    /// 실패 응답 생성
    pub fn failure(result: McsResult, user_id: u16, requested_channel_id: u16) -> Self {
        Self::new(result, user_id, requested_channel_id, None)
    }
}

impl Pdu for ChannelJoinConfirm {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let mut writer = BerWriter::new();

        writer.write_application_tag(MCS_CHANNEL_JOIN_CONFIRM, |w| {
            w.write_enumerated(self.result as u8);
            w.write_integer(self.user_id as u32);
            w.write_integer(self.requested_channel_id as u32);
            if let Some(channel_id) = self.channel_id {
                w.write_integer(channel_id as u32);
            }
        });

        buffer.write_all(writer.as_bytes())?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;

        let mut reader = BerReader::new(&data);
        reader.read_application_tag(MCS_CHANNEL_JOIN_CONFIRM)?;

        let result_code = reader.read_enumerated()?;
        let result = McsResult::from_u8(result_code).ok_or_else(|| {
            PduError::ParseError(format!("Invalid MCS result code: {}", result_code))
        })?;

        let user_id = reader.read_integer()? as u16;
        let requested_channel_id = reader.read_integer()? as u16;

        let channel_id = if reader.remaining() > 0 {
            Some(reader.read_integer()? as u16)
        } else {
            None
        };

        Ok(Self {
            result,
            user_id,
            requested_channel_id,
            channel_id,
        })
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
    fn test_channel_join_request() {
        let request = ChannelJoinRequest::new(1001, 1003);

        let mut buffer = Vec::new();
        request.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ChannelJoinRequest::decode(&mut cursor).unwrap();

        assert_eq!(request, decoded);
        assert_eq!(decoded.user_id, 1001);
        assert_eq!(decoded.channel_id, 1003);
    }

    #[test]
    fn test_channel_join_request_roundtrip() {
        let test_cases = vec![
            ChannelJoinRequest::new(1, 1),
            ChannelJoinRequest::new(1001, 1003),
            ChannelJoinRequest::new(65535, 65535),
        ];

        for request in test_cases {
            let mut buffer = Vec::new();
            request.encode(&mut buffer).unwrap();

            let mut cursor = Cursor::new(buffer);
            let decoded = ChannelJoinRequest::decode(&mut cursor).unwrap();

            assert_eq!(request, decoded);
        }
    }

    #[test]
    fn test_channel_join_confirm_success() {
        let confirm = ChannelJoinConfirm::success(1001, 1003);

        let mut buffer = Vec::new();
        confirm.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ChannelJoinConfirm::decode(&mut cursor).unwrap();

        assert_eq!(confirm, decoded);
        assert_eq!(decoded.result, McsResult::RtSuccessful);
        assert_eq!(decoded.user_id, 1001);
        assert_eq!(decoded.requested_channel_id, 1003);
        assert_eq!(decoded.channel_id, Some(1003));
    }

    #[test]
    fn test_channel_join_confirm_failure() {
        let confirm = ChannelJoinConfirm::failure(McsResult::RtNoSuchChannel, 1001, 1003);

        let mut buffer = Vec::new();
        confirm.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ChannelJoinConfirm::decode(&mut cursor).unwrap();

        assert_eq!(confirm, decoded);
        assert_eq!(decoded.result, McsResult::RtNoSuchChannel);
        assert_eq!(decoded.user_id, 1001);
        assert_eq!(decoded.requested_channel_id, 1003);
        assert_eq!(decoded.channel_id, None);
    }

    #[test]
    fn test_channel_join_confirm_roundtrip() {
        let test_cases = vec![
            ChannelJoinConfirm::success(1, 1),
            ChannelJoinConfirm::success(1001, 1003),
            ChannelJoinConfirm::failure(McsResult::RtNoSuchChannel, 1001, 1003),
            ChannelJoinConfirm::failure(McsResult::RtTooManyChannels, 500, 600),
        ];

        for confirm in test_cases {
            let mut buffer = Vec::new();
            confirm.encode(&mut buffer).unwrap();

            let mut cursor = Cursor::new(buffer);
            let decoded = ChannelJoinConfirm::decode(&mut cursor).unwrap();

            assert_eq!(confirm, decoded);
        }
    }
}
