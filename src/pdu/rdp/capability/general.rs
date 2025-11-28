use crate::pdu::{Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use super::sets::{CapabilitySetHeader, CapabilitySetType};

/// General Capability Set (MS-RDPBCGR 2.2.7.1.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralCapability {
    /// OS major type (1 = Windows)
    pub os_major_type: u16,
    /// OS minor type (various Windows versions)
    pub os_minor_type: u16,
    /// Protocol version (0x0200 for RDP 5.0+)
    pub protocol_version: u16,
    /// General compression types
    pub general_compression_types: u16,
    /// Extra flags
    pub extra_flags: u16,
    /// Update capability flag (0 = not supported, 1 = supported)
    pub update_capability_flag: u16,
    /// Remote unshare flag
    pub remote_unshare_flag: u16,
    /// General compression level
    pub general_compression_level: u16,
    /// Refresh rect support (1 = supported)
    pub refresh_rect_support: u8,
    /// Suppress output support (1 = supported)
    pub suppress_output_support: u8,
}

impl GeneralCapability {
    /// Create default General Capability
    pub fn new() -> Self {
        Self {
            os_major_type: 1,      // Windows
            os_minor_type: 3,      // Windows NT
            protocol_version: 0x0200, // RDP 5.0+
            general_compression_types: 0,
            extra_flags: 0x040D, // FASTPATH_OUTPUT_SUPPORTED | LONG_CREDENTIALS_SUPPORTED | AUTORECONNECT_SUPPORTED | ENC_SALTED_CHECKSUM
            update_capability_flag: 0,
            remote_unshare_flag: 0,
            general_compression_level: 0,
            refresh_rect_support: 1,
            suppress_output_support: 1,
        }
    }

    /// Data size (excluding header)
    pub const DATA_SIZE: usize = 20;

    /// Encode capability set (with header)
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let header = CapabilitySetHeader::new(
            CapabilitySetType::General,
            (CapabilitySetHeader::SIZE + Self::DATA_SIZE) as u16,
        );
        header.encode(buffer)?;

        buffer.write_u16::<LittleEndian>(self.os_major_type)?;
        buffer.write_u16::<LittleEndian>(self.os_minor_type)?;
        buffer.write_u16::<LittleEndian>(self.protocol_version)?;
        buffer.write_u16::<LittleEndian>(0)?; // padding
        buffer.write_u16::<LittleEndian>(self.general_compression_types)?;
        buffer.write_u16::<LittleEndian>(self.extra_flags)?;
        buffer.write_u16::<LittleEndian>(self.update_capability_flag)?;
        buffer.write_u16::<LittleEndian>(self.remote_unshare_flag)?;
        buffer.write_u16::<LittleEndian>(self.general_compression_level)?;
        buffer.write_u8(self.refresh_rect_support)?;
        buffer.write_u8(self.suppress_output_support)?;

        Ok(())
    }

    /// Decode capability data (without header)
    pub fn decode_data(buffer: &mut dyn Read, _data_len: usize) -> Result<Self> {
        let os_major_type = buffer.read_u16::<LittleEndian>()?;
        let os_minor_type = buffer.read_u16::<LittleEndian>()?;
        let protocol_version = buffer.read_u16::<LittleEndian>()?;
        let _padding = buffer.read_u16::<LittleEndian>()?;
        let general_compression_types = buffer.read_u16::<LittleEndian>()?;
        let extra_flags = buffer.read_u16::<LittleEndian>()?;
        let update_capability_flag = buffer.read_u16::<LittleEndian>()?;
        let remote_unshare_flag = buffer.read_u16::<LittleEndian>()?;
        let general_compression_level = buffer.read_u16::<LittleEndian>()?;
        let refresh_rect_support = buffer.read_u8()?;
        let suppress_output_support = buffer.read_u8()?;

        Ok(Self {
            os_major_type,
            os_minor_type,
            protocol_version,
            general_compression_types,
            extra_flags,
            update_capability_flag,
            remote_unshare_flag,
            general_compression_level,
            refresh_rect_support,
            suppress_output_support,
        })
    }

    /// Get size (including header)
    pub fn size(&self) -> usize {
        CapabilitySetHeader::SIZE + Self::DATA_SIZE
    }
}

impl Default for GeneralCapability {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_general_capability() {
        let cap = GeneralCapability::new();

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), cap.size());

        let mut cursor = Cursor::new(&buffer[CapabilitySetHeader::SIZE..]);
        let decoded = GeneralCapability::decode_data(&mut cursor, GeneralCapability::DATA_SIZE).unwrap();

        assert_eq!(decoded.protocol_version, 0x0200);
        assert_eq!(decoded.os_major_type, 1);
        assert_eq!(decoded.refresh_rect_support, 1);
    }

    #[test]
    fn test_general_capability_roundtrip() {
        let cap = GeneralCapability::new();

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(&buffer[CapabilitySetHeader::SIZE..]);
        let decoded = GeneralCapability::decode_data(&mut cursor, GeneralCapability::DATA_SIZE).unwrap();

        assert_eq!(cap, decoded);
    }
}
