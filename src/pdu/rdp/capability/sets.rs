use crate::pdu::{PduError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use super::{BitmapCapability, GeneralCapability, InputCapability, OrderCapability};

/// Capability Set Type (MS-RDPBCGR 2.2.1.13.1.1.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum CapabilitySetType {
    /// General Capability Set
    General = 0x0001,
    /// Bitmap Capability Set
    Bitmap = 0x0002,
    /// Order Capability Set
    Order = 0x0003,
    /// Bitmap Cache Capability Set
    BitmapCache = 0x0004,
    /// Control Capability Set
    Control = 0x0005,
    /// Activation Capability Set
    Activation = 0x0007,
    /// Pointer Capability Set
    Pointer = 0x0008,
    /// Share Capability Set
    Share = 0x0009,
    /// Color Cache Capability Set
    ColorCache = 0x000A,
    /// Sound Capability Set
    Sound = 0x000C,
    /// Input Capability Set
    Input = 0x000D,
    /// Font Capability Set
    Font = 0x000E,
    /// Brush Capability Set
    Brush = 0x000F,
    /// Glyph Cache Capability Set
    GlyphCache = 0x0010,
    /// Offscreen Bitmap Cache Capability Set
    OffscreenCache = 0x0011,
    /// Bitmap Cache Host Support Capability Set
    BitmapCacheHostSupport = 0x0012,
    /// Bitmap Cache V2 Capability Set
    BitmapCacheV2 = 0x0013,
    /// Virtual Channel Capability Set
    VirtualChannel = 0x0014,
    /// DrawNineGrid Capability Set
    DrawNineGrid = 0x0015,
    /// Draw GDI+ Capability Set
    DrawGdiPlus = 0x0016,
    /// Remote Programs Capability Set
    Rail = 0x0017,
    /// Window List Capability Set
    Window = 0x0018,
    /// Desktop Composition Capability Set
    DesktopComposition = 0x0019,
    /// Multifragment Update Capability Set
    MultifragmentUpdate = 0x001A,
    /// Large Pointer Capability Set
    LargePointer = 0x001B,
    /// Surface Commands Capability Set
    SurfaceCommands = 0x001C,
    /// Bitmap Codecs Capability Set
    BitmapCodecs = 0x001D,
    /// Frame Acknowledge Capability Set
    FrameAcknowledge = 0x001E,
}

impl CapabilitySetType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0001 => Some(CapabilitySetType::General),
            0x0002 => Some(CapabilitySetType::Bitmap),
            0x0003 => Some(CapabilitySetType::Order),
            0x0004 => Some(CapabilitySetType::BitmapCache),
            0x0005 => Some(CapabilitySetType::Control),
            0x0007 => Some(CapabilitySetType::Activation),
            0x0008 => Some(CapabilitySetType::Pointer),
            0x0009 => Some(CapabilitySetType::Share),
            0x000A => Some(CapabilitySetType::ColorCache),
            0x000C => Some(CapabilitySetType::Sound),
            0x000D => Some(CapabilitySetType::Input),
            0x000E => Some(CapabilitySetType::Font),
            0x000F => Some(CapabilitySetType::Brush),
            0x0010 => Some(CapabilitySetType::GlyphCache),
            0x0011 => Some(CapabilitySetType::OffscreenCache),
            0x0012 => Some(CapabilitySetType::BitmapCacheHostSupport),
            0x0013 => Some(CapabilitySetType::BitmapCacheV2),
            0x0014 => Some(CapabilitySetType::VirtualChannel),
            0x0015 => Some(CapabilitySetType::DrawNineGrid),
            0x0016 => Some(CapabilitySetType::DrawGdiPlus),
            0x0017 => Some(CapabilitySetType::Rail),
            0x0018 => Some(CapabilitySetType::Window),
            0x0019 => Some(CapabilitySetType::DesktopComposition),
            0x001A => Some(CapabilitySetType::MultifragmentUpdate),
            0x001B => Some(CapabilitySetType::LargePointer),
            0x001C => Some(CapabilitySetType::SurfaceCommands),
            0x001D => Some(CapabilitySetType::BitmapCodecs),
            0x001E => Some(CapabilitySetType::FrameAcknowledge),
            _ => None,
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Capability Set Header (MS-RDPBCGR 2.2.1.13.1.1.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySetHeader {
    /// Capability set type
    pub capability_set_type: CapabilitySetType,
    /// Length of capability data (including header)
    pub length_capability: u16,
}

impl CapabilitySetHeader {
    /// Header size (4 bytes)
    pub const SIZE: usize = 4;

    /// Create new capability set header
    pub fn new(capability_set_type: CapabilitySetType, length_capability: u16) -> Self {
        Self {
            capability_set_type,
            length_capability,
        }
    }

    /// Encode header
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.capability_set_type.as_u16())?;
        buffer.write_u16::<LittleEndian>(self.length_capability)?;
        Ok(())
    }

    /// Decode header
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let type_value = buffer.read_u16::<LittleEndian>()?;
        let capability_set_type = CapabilitySetType::from_u16(type_value).ok_or_else(|| {
            PduError::ParseError(format!("Invalid capability set type: {:#x}", type_value))
        })?;

        let length_capability = buffer.read_u16::<LittleEndian>()?;

        Ok(Self {
            capability_set_type,
            length_capability,
        })
    }
}

/// Capability Set (enum of all capability types)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilitySet {
    /// General Capability Set
    General(GeneralCapability),
    /// Bitmap Capability Set
    Bitmap(BitmapCapability),
    /// Order Capability Set
    Order(OrderCapability),
    /// Input Capability Set
    Input(InputCapability),
    /// Unknown/Unsupported capability set (type, data)
    Unknown(u16, Vec<u8>),
}

impl CapabilitySet {
    /// Get capability set type
    pub fn capability_type(&self) -> CapabilitySetType {
        match self {
            CapabilitySet::General(_) => CapabilitySetType::General,
            CapabilitySet::Bitmap(_) => CapabilitySetType::Bitmap,
            CapabilitySet::Order(_) => CapabilitySetType::Order,
            CapabilitySet::Input(_) => CapabilitySetType::Input,
            CapabilitySet::Unknown(type_val, _) => {
                CapabilitySetType::from_u16(*type_val).unwrap_or(CapabilitySetType::General)
            }
        }
    }

    /// Encode capability set (with header)
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        match self {
            CapabilitySet::General(cap) => cap.encode(buffer),
            CapabilitySet::Bitmap(cap) => cap.encode(buffer),
            CapabilitySet::Order(cap) => cap.encode(buffer),
            CapabilitySet::Input(cap) => cap.encode(buffer),
            CapabilitySet::Unknown(type_val, data) => {
                let header = CapabilitySetHeader::new(
                    CapabilitySetType::from_u16(*type_val).unwrap_or(CapabilitySetType::General),
                    (CapabilitySetHeader::SIZE + data.len()) as u16,
                );
                header.encode(buffer)?;
                buffer.write_all(data)?;
                Ok(())
            }
        }
    }

    /// Decode capability set from buffer
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let header = CapabilitySetHeader::decode(buffer)?;
        let data_len = header.length_capability as usize - CapabilitySetHeader::SIZE;

        match header.capability_set_type {
            CapabilitySetType::General => {
                let cap = GeneralCapability::decode_data(buffer, data_len)?;
                Ok(CapabilitySet::General(cap))
            }
            CapabilitySetType::Bitmap => {
                let cap = BitmapCapability::decode_data(buffer, data_len)?;
                Ok(CapabilitySet::Bitmap(cap))
            }
            CapabilitySetType::Order => {
                let cap = OrderCapability::decode_data(buffer, data_len)?;
                Ok(CapabilitySet::Order(cap))
            }
            CapabilitySetType::Input => {
                let cap = InputCapability::decode_data(buffer, data_len)?;
                Ok(CapabilitySet::Input(cap))
            }
            _ => {
                // Unknown capability - read as raw data
                let mut data = vec![0u8; data_len];
                buffer.read_exact(&mut data)?;
                Ok(CapabilitySet::Unknown(
                    header.capability_set_type.as_u16(),
                    data,
                ))
            }
        }
    }

    /// Get size of capability set (including header)
    pub fn size(&self) -> usize {
        match self {
            CapabilitySet::General(cap) => cap.size(),
            CapabilitySet::Bitmap(cap) => cap.size(),
            CapabilitySet::Order(cap) => cap.size(),
            CapabilitySet::Input(cap) => cap.size(),
            CapabilitySet::Unknown(_, data) => CapabilitySetHeader::SIZE + data.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_capability_set_type() {
        assert_eq!(CapabilitySetType::General.as_u16(), 0x0001);
        assert_eq!(
            CapabilitySetType::from_u16(0x0001),
            Some(CapabilitySetType::General)
        );
        assert_eq!(CapabilitySetType::from_u16(0xFFFF), None);
    }

    #[test]
    fn test_capability_set_header() {
        let header = CapabilitySetHeader::new(CapabilitySetType::General, 24);

        let mut buffer = Vec::new();
        header.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), CapabilitySetHeader::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = CapabilitySetHeader::decode(&mut cursor).unwrap();

        assert_eq!(decoded.capability_set_type, CapabilitySetType::General);
        assert_eq!(decoded.length_capability, 24);
    }

    #[test]
    fn test_unknown_capability_set() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let cap = CapabilitySet::Unknown(0x00FF, data.clone());

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        assert_eq!(cap.size(), CapabilitySetHeader::SIZE + data.len());
    }
}
