use crate::codec::{BerReader, BerWriter};
use crate::pdu::{Pdu, PduError, Result};
use std::io::{Read, Write};

use super::domain::McsResult;

/// MCS Connect-Initial TAG ([APPLICATION 101])
pub const MCS_CONNECT_INITIAL: u8 = 101;

/// MCS Connect-Response TAG ([APPLICATION 102])
pub const MCS_CONNECT_RESPONSE: u8 = 102;

/// MCS Domain Parameters
///
/// DomainParameters ::= SEQUENCE {
///     maxChannelIds    INTEGER (0..MAX),
///     maxUserIds       INTEGER (0..MAX),
///     maxTokenIds      INTEGER (0..MAX),
///     numPriorities    INTEGER (0..MAX),
///     minThroughput    INTEGER (0..MAX),
///     maxHeight        INTEGER (0..MAX),
///     maxMCSPDUsize    INTEGER (0..MAX),
///     protocolVersion  INTEGER (0..MAX)
/// }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainParameters {
    pub max_channel_ids: u32,
    pub max_user_ids: u32,
    pub max_token_ids: u32,
    pub num_priorities: u32,
    pub min_throughput: u32,
    pub max_height: u32,
    pub max_mcspdu_size: u32,
    pub protocol_version: u32,
}

impl DomainParameters {
    /// 새로운 DomainParameters 생성
    pub fn new(
        max_channel_ids: u32,
        max_user_ids: u32,
        max_token_ids: u32,
        num_priorities: u32,
        min_throughput: u32,
        max_height: u32,
        max_mcspdu_size: u32,
        protocol_version: u32,
    ) -> Self {
        Self {
            max_channel_ids,
            max_user_ids,
            max_token_ids,
            num_priorities,
            min_throughput,
            max_height,
            max_mcspdu_size,
            protocol_version,
        }
    }

    /// RDP 클라이언트 기본 target parameters
    pub fn target() -> Self {
        Self {
            max_channel_ids: 34,
            max_user_ids: 2,
            max_token_ids: 0,
            num_priorities: 1,
            min_throughput: 0,
            max_height: 1,
            max_mcspdu_size: 65535,
            protocol_version: 2,
        }
    }

    /// RDP 클라이언트 기본 minimum parameters
    pub fn minimum() -> Self {
        Self {
            max_channel_ids: 1,
            max_user_ids: 1,
            max_token_ids: 1,
            num_priorities: 1,
            min_throughput: 0,
            max_height: 1,
            max_mcspdu_size: 1056,
            protocol_version: 2,
        }
    }

    /// RDP 클라이언트 기본 maximum parameters
    pub fn maximum() -> Self {
        Self {
            max_channel_ids: 65535,
            max_user_ids: 64535,
            max_token_ids: 65535,
            num_priorities: 1,
            min_throughput: 0,
            max_height: 1,
            max_mcspdu_size: 65535,
            protocol_version: 2,
        }
    }

    /// BER 인코딩
    pub fn encode(&self, writer: &mut BerWriter) {
        writer.write_sequence(|w| {
            w.write_integer(self.max_channel_ids);
            w.write_integer(self.max_user_ids);
            w.write_integer(self.max_token_ids);
            w.write_integer(self.num_priorities);
            w.write_integer(self.min_throughput);
            w.write_integer(self.max_height);
            w.write_integer(self.max_mcspdu_size);
            w.write_integer(self.protocol_version);
        });
    }

    /// BER 디코딩
    pub fn decode(reader: &mut BerReader) -> Result<Self> {
        reader.read_tag()?; // SEQUENCE tag
        reader.read_length()?;

        Ok(Self {
            max_channel_ids: reader.read_integer()?,
            max_user_ids: reader.read_integer()?,
            max_token_ids: reader.read_integer()?,
            num_priorities: reader.read_integer()?,
            min_throughput: reader.read_integer()?,
            max_height: reader.read_integer()?,
            max_mcspdu_size: reader.read_integer()?,
            protocol_version: reader.read_integer()?,
        })
    }
}

impl Default for DomainParameters {
    fn default() -> Self {
        Self::target()
    }
}

/// MCS Connect-Initial
///
/// Connect-Initial ::= [APPLICATION 101] IMPLICIT SEQUENCE {
///     callingDomainSelector    OCTET STRING,
///     calledDomainSelector     OCTET STRING,
///     upwardFlag               BOOLEAN,
///     targetParameters         DomainParameters,
///     minimumParameters        DomainParameters,
///     maximumParameters        DomainParameters,
///     userData                 OCTET STRING
/// }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectInitial {
    pub calling_domain: Vec<u8>,
    pub called_domain: Vec<u8>,
    pub upward_flag: bool,
    pub target_parameters: DomainParameters,
    pub minimum_parameters: DomainParameters,
    pub maximum_parameters: DomainParameters,
    pub user_data: Vec<u8>,
}

impl ConnectInitial {
    /// 새로운 ConnectInitial 생성
    pub fn new(user_data: Vec<u8>) -> Self {
        Self {
            calling_domain: vec![1],
            called_domain: vec![1],
            upward_flag: true,
            target_parameters: DomainParameters::target(),
            minimum_parameters: DomainParameters::minimum(),
            maximum_parameters: DomainParameters::maximum(),
            user_data,
        }
    }

    /// 커스텀 파라미터로 생성
    pub fn with_parameters(
        target: DomainParameters,
        minimum: DomainParameters,
        maximum: DomainParameters,
        user_data: Vec<u8>,
    ) -> Self {
        Self {
            calling_domain: vec![1],
            called_domain: vec![1],
            upward_flag: true,
            target_parameters: target,
            minimum_parameters: minimum,
            maximum_parameters: maximum,
            user_data,
        }
    }
}

impl Pdu for ConnectInitial {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let mut writer = BerWriter::new();

        writer.write_application_tag(MCS_CONNECT_INITIAL, |w| {
            w.write_octet_string(&self.calling_domain);
            w.write_octet_string(&self.called_domain);
            w.write_boolean(self.upward_flag);

            self.target_parameters.encode(w);
            self.minimum_parameters.encode(w);
            self.maximum_parameters.encode(w);

            w.write_octet_string(&self.user_data);
        });

        buffer.write_all(writer.as_bytes())?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;

        let mut reader = BerReader::new(&data);
        reader.read_application_tag(MCS_CONNECT_INITIAL)?;

        let calling_domain = reader.read_octet_string()?;
        let called_domain = reader.read_octet_string()?;
        let upward_flag = reader.read_boolean()?;

        let target_parameters = DomainParameters::decode(&mut reader)?;
        let minimum_parameters = DomainParameters::decode(&mut reader)?;
        let maximum_parameters = DomainParameters::decode(&mut reader)?;

        let user_data = reader.read_octet_string()?;

        Ok(Self {
            calling_domain,
            called_domain,
            upward_flag,
            target_parameters,
            minimum_parameters,
            maximum_parameters,
            user_data,
        })
    }

    fn size(&self) -> usize {
        let mut writer = BerWriter::new();
        self.encode(&mut writer).unwrap();
        writer.as_bytes().len()
    }
}

/// MCS Connect-Response
///
/// Connect-Response ::= [APPLICATION 102] IMPLICIT SEQUENCE {
///     result                   Result,
///     calledConnectId          INTEGER,
///     domainParameters         DomainParameters,
///     userData                 OCTET STRING
/// }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectResponse {
    pub result: McsResult,
    pub called_connect_id: u32,
    pub domain_parameters: DomainParameters,
    pub user_data: Vec<u8>,
}

impl ConnectResponse {
    /// 새로운 ConnectResponse 생성
    pub fn new(
        result: McsResult,
        called_connect_id: u32,
        domain_parameters: DomainParameters,
        user_data: Vec<u8>,
    ) -> Self {
        Self {
            result,
            called_connect_id,
            domain_parameters,
            user_data,
        }
    }

    /// 성공 응답 생성
    pub fn success(user_data: Vec<u8>) -> Self {
        Self {
            result: McsResult::RtSuccessful,
            called_connect_id: 0,
            domain_parameters: DomainParameters::target(),
            user_data,
        }
    }
}

impl Pdu for ConnectResponse {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let mut writer = BerWriter::new();

        writer.write_application_tag(MCS_CONNECT_RESPONSE, |w| {
            w.write_enumerated(self.result as u8);
            w.write_integer(self.called_connect_id);
            self.domain_parameters.encode(w);
            w.write_octet_string(&self.user_data);
        });

        buffer.write_all(writer.as_bytes())?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let mut data = Vec::new();
        buffer.read_to_end(&mut data)?;

        let mut reader = BerReader::new(&data);
        reader.read_application_tag(MCS_CONNECT_RESPONSE)?;

        let result_code = reader.read_enumerated()?;
        let result = McsResult::from_u8(result_code).ok_or_else(|| {
            PduError::ParseError(format!("Invalid MCS result code: {}", result_code))
        })?;

        let called_connect_id = reader.read_integer()?;
        let domain_parameters = DomainParameters::decode(&mut reader)?;
        let user_data = reader.read_octet_string()?;

        Ok(Self {
            result,
            called_connect_id,
            domain_parameters,
            user_data,
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
    fn test_domain_parameters() {
        let params = DomainParameters::target();
        assert_eq!(params.max_channel_ids, 34);
        assert_eq!(params.protocol_version, 2);

        let min = DomainParameters::minimum();
        assert_eq!(min.max_channel_ids, 1);

        let max = DomainParameters::maximum();
        assert_eq!(max.max_channel_ids, 65535);
    }

    #[test]
    fn test_domain_parameters_encode_decode() {
        let params = DomainParameters::target();
        let mut writer = BerWriter::new();
        params.encode(&mut writer);

        let mut reader = BerReader::new(writer.as_bytes());
        let decoded = DomainParameters::decode(&mut reader).unwrap();

        assert_eq!(params, decoded);
    }

    #[test]
    fn test_connect_initial() {
        let user_data = b"test_gcc_data".to_vec();
        let initial = ConnectInitial::new(user_data.clone());

        let mut buffer = Vec::new();
        initial.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectInitial::decode(&mut cursor).unwrap();

        assert_eq!(initial.user_data, decoded.user_data);
        assert_eq!(initial.upward_flag, decoded.upward_flag);
        assert_eq!(
            initial.target_parameters.max_channel_ids,
            decoded.target_parameters.max_channel_ids
        );
    }

    #[test]
    fn test_connect_initial_with_custom_parameters() {
        let target = DomainParameters::target();
        let minimum = DomainParameters::minimum();
        let maximum = DomainParameters::maximum();
        let user_data = b"custom_data".to_vec();

        let initial = ConnectInitial::with_parameters(target.clone(), minimum, maximum, user_data);

        let mut buffer = Vec::new();
        initial.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectInitial::decode(&mut cursor).unwrap();

        assert_eq!(
            initial.target_parameters.max_channel_ids,
            decoded.target_parameters.max_channel_ids
        );
    }

    #[test]
    fn test_connect_response() {
        let user_data = b"response_data".to_vec();
        let response = ConnectResponse::success(user_data.clone());

        let mut buffer = Vec::new();
        response.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectResponse::decode(&mut cursor).unwrap();

        assert_eq!(response.result, decoded.result);
        assert_eq!(response.user_data, decoded.user_data);
        assert_eq!(
            response.domain_parameters.max_channel_ids,
            decoded.domain_parameters.max_channel_ids
        );
    }

    #[test]
    fn test_connect_response_custom() {
        let params = DomainParameters::target();
        let user_data = b"test".to_vec();
        let response = ConnectResponse::new(McsResult::RtSuccessful, 42, params, user_data);

        let mut buffer = Vec::new();
        response.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ConnectResponse::decode(&mut cursor).unwrap();

        assert_eq!(response.called_connect_id, decoded.called_connect_id);
        assert_eq!(response.result, decoded.result);
    }
}
