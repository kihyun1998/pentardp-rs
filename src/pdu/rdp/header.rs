use crate::pdu::{PduError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// RDP PDU Type (Share Control Header)
///
/// OR'd with TS_PROTOCOL_VERSION (0x10) during transmission
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PduType {
    /// Demand Active PDU (Server -> Client)
    DemandActive = 0x01,
    /// Confirm Active PDU (Client -> Server)
    ConfirmActive = 0x03,
    /// Deactivate All PDU
    DeactivateAll = 0x06,
    /// Data PDU (Bidirectional)
    Data = 0x07,
    /// Server Redirect PDU
    ServerRedirect = 0x0A,
}

impl PduType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x01 => Some(PduType::DemandActive),
            0x03 => Some(PduType::ConfirmActive),
            0x06 => Some(PduType::DeactivateAll),
            0x07 => Some(PduType::Data),
            0x0A => Some(PduType::ServerRedirect),
            _ => None,
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// RDP Data PDU Subtype (Share Data Header)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DataPduType {
    /// Update PDU
    Update = 0x02,
    /// Control PDU
    Control = 0x14,
    /// Pointer PDU
    Pointer = 0x1B,
    /// Input PDU
    Input = 0x1C,
    /// Synchronize PDU
    Synchronize = 0x1F,
    /// Refresh Rect PDU
    RefreshRect = 0x21,
    /// Play Sound PDU
    PlaySound = 0x22,
    /// Suppress Output PDU
    SuppressOutput = 0x23,
    /// Shutdown Request PDU
    ShutdownRequest = 0x24,
    /// Shutdown Denied PDU
    ShutdownDenied = 0x25,
    /// Save Session Info PDU
    SaveSessionInfo = 0x26,
    /// Font List PDU
    FontList = 0x27,
    /// Font Map PDU
    FontMap = 0x28,
    /// Set Keyboard Indicators PDU
    SetKeyboardIndicators = 0x29,
    /// Bitmap Cache Persistent List PDU
    BitmapCachePersistentList = 0x2B,
    /// Bitmap Cache Error PDU
    BitmapCacheError = 0x2C,
    /// Set Keyboard IME Status PDU
    SetKeyboardImeStatus = 0x2D,
    /// Offscreen Cache Error PDU
    OffscreenCacheError = 0x2E,
    /// Set Error Info PDU
    SetErrorInfo = 0x2F,
    /// Draw Nine Grid Error PDU
    DrawNineGridError = 0x30,
    /// Draw GDI+ Error PDU
    DrawGdiPlusError = 0x31,
    /// ARC Status PDU
    ArcStatus = 0x32,
    /// Status Info PDU
    StatusInfo = 0x36,
    /// Monitor Layout PDU
    MonitorLayout = 0x37,
}

impl DataPduType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x02 => Some(DataPduType::Update),
            0x14 => Some(DataPduType::Control),
            0x1B => Some(DataPduType::Pointer),
            0x1C => Some(DataPduType::Input),
            0x1F => Some(DataPduType::Synchronize),
            0x21 => Some(DataPduType::RefreshRect),
            0x22 => Some(DataPduType::PlaySound),
            0x23 => Some(DataPduType::SuppressOutput),
            0x24 => Some(DataPduType::ShutdownRequest),
            0x25 => Some(DataPduType::ShutdownDenied),
            0x26 => Some(DataPduType::SaveSessionInfo),
            0x27 => Some(DataPduType::FontList),
            0x28 => Some(DataPduType::FontMap),
            0x29 => Some(DataPduType::SetKeyboardIndicators),
            0x2B => Some(DataPduType::BitmapCachePersistentList),
            0x2C => Some(DataPduType::BitmapCacheError),
            0x2D => Some(DataPduType::SetKeyboardImeStatus),
            0x2E => Some(DataPduType::OffscreenCacheError),
            0x2F => Some(DataPduType::SetErrorInfo),
            0x30 => Some(DataPduType::DrawNineGridError),
            0x31 => Some(DataPduType::DrawGdiPlusError),
            0x32 => Some(DataPduType::ArcStatus),
            0x36 => Some(DataPduType::StatusInfo),
            0x37 => Some(DataPduType::MonitorLayout),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Share Control Header
///
/// Base header for all RDP PDUs
///
/// Structure:
/// - totalLength (2 bytes): Total PDU length
/// - pduType (2 bytes): PDU type | 0x0010 (TS_PROTOCOL_VERSION)
/// - pduSource (2 bytes): PDU source MCS channel ID
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareControlHeader {
    /// Total PDU length (including this header)
    pub total_length: u16,
    /// PDU Type
    pub pdu_type: PduType,
    /// PDU Source (MCS Channel ID)
    pub pdu_source: u16,
}

impl ShareControlHeader {
    /// Create new Share Control Header
    pub fn new(total_length: u16, pdu_type: PduType, pdu_source: u16) -> Self {
        Self {
            total_length,
            pdu_type,
            pdu_source,
        }
    }

    /// Header size (6 bytes)
    pub const SIZE: usize = 6;

    /// TS_PROTOCOL_VERSION flag (always OR'd to pduType)
    pub const PROTOCOL_VERSION: u16 = 0x0010;

    /// Encode
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.total_length)?;
        buffer.write_u16::<LittleEndian>(self.pdu_type.as_u16() | Self::PROTOCOL_VERSION)?;
        buffer.write_u16::<LittleEndian>(self.pdu_source)?;
        Ok(())
    }

    /// Decode
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let total_length = buffer.read_u16::<LittleEndian>()?;
        let pdu_type_raw = buffer.read_u16::<LittleEndian>()?;
        let pdu_source = buffer.read_u16::<LittleEndian>()?;

        // Remove TS_PROTOCOL_VERSION flag
        let pdu_type_value = pdu_type_raw & !Self::PROTOCOL_VERSION;
        let pdu_type = PduType::from_u16(pdu_type_value).ok_or_else(|| {
            PduError::ParseError(format!("Invalid PDU type: {:#x}", pdu_type_value))
        })?;

        Ok(Self {
            total_length,
            pdu_type,
            pdu_source,
        })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

/// Share Data Header
///
/// Sub-header for Data PDU (PduType::Data)
///
/// Structure:
/// - shareId (4 bytes): Share ID
/// - pad1 (1 byte): Padding (0)
/// - streamId (1 byte): Stream ID (usually 1)
/// - uncompressedLength (2 bytes): Length before compression
/// - pduType2 (1 byte): Data PDU subtype
/// - compressedType (1 byte): Compression type and flags
/// - compressedLength (2 bytes): Compressed length
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareDataHeader {
    /// Share ID
    pub share_id: u32,
    /// Stream ID (usually 1: STREAM_LOW)
    pub stream_id: u8,
    /// Length before compression
    pub uncompressed_length: u16,
    /// Data PDU subtype
    pub pdu_type2: DataPduType,
    /// Compression type and flags
    pub compressed_type: u8,
    /// Compressed data length
    pub compressed_length: u16,
}

impl ShareDataHeader {
    /// Create new Share Data Header (no compression)
    pub fn new(share_id: u32, pdu_type2: DataPduType, uncompressed_length: u16) -> Self {
        Self {
            share_id,
            stream_id: 1, // STREAM_LOW
            uncompressed_length,
            pdu_type2,
            compressed_type: 0,
            compressed_length: 0,
        }
    }

    /// Header size (12 bytes)
    pub const SIZE: usize = 12;

    /// Stream ID: STREAM_LOW
    pub const STREAM_LOW: u8 = 1;
    /// Stream ID: STREAM_MED
    pub const STREAM_MED: u8 = 2;
    /// Stream ID: STREAM_HI
    pub const STREAM_HI: u8 = 4;

    /// Encode
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u32::<LittleEndian>(self.share_id)?;
        buffer.write_u8(0)?; // pad1
        buffer.write_u8(self.stream_id)?;
        buffer.write_u16::<LittleEndian>(self.uncompressed_length)?;
        buffer.write_u8(self.pdu_type2.as_u8())?;
        buffer.write_u8(self.compressed_type)?;
        buffer.write_u16::<LittleEndian>(self.compressed_length)?;
        Ok(())
    }

    /// Decode
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let share_id = buffer.read_u32::<LittleEndian>()?;
        let _pad1 = buffer.read_u8()?; // padding, ignored
        let stream_id = buffer.read_u8()?;
        let uncompressed_length = buffer.read_u16::<LittleEndian>()?;
        let pdu_type2_value = buffer.read_u8()?;
        let compressed_type = buffer.read_u8()?;
        let compressed_length = buffer.read_u16::<LittleEndian>()?;

        let pdu_type2 = DataPduType::from_u8(pdu_type2_value).ok_or_else(|| {
            PduError::ParseError(format!("Invalid Data PDU type: {:#x}", pdu_type2_value))
        })?;

        Ok(Self {
            share_id,
            stream_id,
            uncompressed_length,
            pdu_type2,
            compressed_type,
            compressed_length,
        })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_pdu_type() {
        assert_eq!(PduType::Data.as_u16(), 0x07);
        assert_eq!(PduType::from_u16(0x07), Some(PduType::Data));
        assert_eq!(PduType::from_u16(0xFF), None);
    }

    #[test]
    fn test_data_pdu_type() {
        assert_eq!(DataPduType::Synchronize.as_u8(), 0x1F);
        assert_eq!(DataPduType::from_u8(0x1F), Some(DataPduType::Synchronize));
        assert_eq!(DataPduType::from_u8(0xFF), None);
    }

    #[test]
    fn test_share_control_header_encode_decode() {
        let header = ShareControlHeader::new(100, PduType::Data, 1004);

        let mut buffer = Vec::new();
        header.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), ShareControlHeader::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = ShareControlHeader::decode(&mut cursor).unwrap();

        assert_eq!(header, decoded);
        assert_eq!(decoded.total_length, 100);
        assert_eq!(decoded.pdu_type, PduType::Data);
        assert_eq!(decoded.pdu_source, 1004);
    }

    #[test]
    fn test_share_control_header_protocol_version() {
        let header = ShareControlHeader::new(50, PduType::ConfirmActive, 1003);

        let mut buffer = Vec::new();
        header.encode(&mut buffer).unwrap();

        // pduType field is 0x0013 | 0x0010 = 0x0013
        let mut cursor = Cursor::new(&buffer);
        cursor.read_u16::<LittleEndian>().unwrap(); // total_length
        let pdu_type_raw = cursor.read_u16::<LittleEndian>().unwrap();

        // TS_PROTOCOL_VERSION flag must be set
        assert_eq!(
            pdu_type_raw & ShareControlHeader::PROTOCOL_VERSION,
            ShareControlHeader::PROTOCOL_VERSION
        );
    }

    #[test]
    fn test_share_data_header_encode_decode() {
        let header = ShareDataHeader::new(0x000103EA, DataPduType::Synchronize, 4);

        let mut buffer = Vec::new();
        header.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), ShareDataHeader::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = ShareDataHeader::decode(&mut cursor).unwrap();

        assert_eq!(header, decoded);
        assert_eq!(decoded.share_id, 0x000103EA);
        assert_eq!(decoded.pdu_type2, DataPduType::Synchronize);
        assert_eq!(decoded.stream_id, 1);
        assert_eq!(decoded.uncompressed_length, 4);
        assert_eq!(decoded.compressed_type, 0);
        assert_eq!(decoded.compressed_length, 0);
    }

    #[test]
    fn test_share_data_header_roundtrip() {
        let header = ShareDataHeader {
            share_id: 0x12345678,
            stream_id: ShareDataHeader::STREAM_MED,
            uncompressed_length: 256,
            pdu_type2: DataPduType::Control,
            compressed_type: 0,
            compressed_length: 0,
        };

        let mut buffer = Vec::new();
        header.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ShareDataHeader::decode(&mut cursor).unwrap();

        assert_eq!(header, decoded);
    }

    #[test]
    fn test_share_data_header_size() {
        let header = ShareDataHeader::new(0, DataPduType::FontList, 0);
        assert_eq!(header.size(), 12);
    }

    #[test]
    fn test_share_control_header_size() {
        let header = ShareControlHeader::new(0, PduType::Data, 0);
        assert_eq!(header.size(), 6);
    }
}
