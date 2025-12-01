// RDP Graphics Update PDUs
pub mod bitmap;
pub mod orders;

pub use bitmap::{BitmapData, BitmapFlags, BitmapUpdate};
pub use orders::{
    DstBltOrder, MemBltOrder, OpaqueRectOrder, OrdersUpdate, OrderType, PatBltOrder, ScrBltOrder,
};

use crate::pdu::{Pdu, PduError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Update Type (MS-RDPBCGR 2.2.9.1.1.3.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum UpdateType {
    /// Orders update (drawing orders)
    Orders = 0x0000,
    /// Bitmap update
    Bitmap = 0x0001,
    /// Palette update
    Palette = 0x0002,
    /// Synchronize update
    Synchronize = 0x0003,
}

impl UpdateType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0000 => Some(UpdateType::Orders),
            0x0001 => Some(UpdateType::Bitmap),
            0x0002 => Some(UpdateType::Palette),
            0x0003 => Some(UpdateType::Synchronize),
            _ => None,
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Update PDU (MS-RDPBCGR 2.2.9.1.1.3)
///
/// Container for various types of graphics updates
///
/// This PDU is sent within a Share Data Header (pdu_type2 = Update)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatePdu {
    /// Orders update (drawing commands)
    Orders(OrdersUpdate),
    /// Bitmap update (pixel data)
    Bitmap(BitmapUpdate),
    /// Palette update (color table)
    Palette(PaletteUpdate),
    /// Synchronize update
    Synchronize,
}

impl UpdatePdu {
    /// Get update type
    pub fn update_type(&self) -> UpdateType {
        match self {
            UpdatePdu::Orders(_) => UpdateType::Orders,
            UpdatePdu::Bitmap(_) => UpdateType::Bitmap,
            UpdatePdu::Palette(_) => UpdateType::Palette,
            UpdatePdu::Synchronize => UpdateType::Synchronize,
        }
    }
}

impl Pdu for UpdatePdu {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        // updateType (2 bytes)
        buffer.write_u16::<LittleEndian>(self.update_type().as_u16())?;

        // Update-specific data
        match self {
            UpdatePdu::Orders(orders) => orders.encode(buffer)?,
            UpdatePdu::Bitmap(bitmap) => bitmap.encode(buffer)?,
            UpdatePdu::Palette(palette) => palette.encode(buffer)?,
            UpdatePdu::Synchronize => {
                // Synchronize has no additional data (2 bytes padding)
                buffer.write_u16::<LittleEndian>(0)?;
            }
        }

        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let update_type = buffer.read_u16::<LittleEndian>()?;

        let update_type = UpdateType::from_u16(update_type).ok_or_else(|| {
            PduError::ParseError(format!("Invalid update type: {:#x}", update_type))
        })?;

        match update_type {
            UpdateType::Orders => Ok(UpdatePdu::Orders(OrdersUpdate::decode(buffer)?)),
            UpdateType::Bitmap => Ok(UpdatePdu::Bitmap(BitmapUpdate::decode(buffer)?)),
            UpdateType::Palette => Ok(UpdatePdu::Palette(PaletteUpdate::decode(buffer)?)),
            UpdateType::Synchronize => {
                // Skip padding
                let _pad = buffer.read_u16::<LittleEndian>()?;
                Ok(UpdatePdu::Synchronize)
            }
        }
    }

    fn size(&self) -> usize {
        2 + match self {
            UpdatePdu::Orders(orders) => orders.size(),
            UpdatePdu::Bitmap(bitmap) => bitmap.size(),
            UpdatePdu::Palette(palette) => palette.size(),
            UpdatePdu::Synchronize => 2,
        }
    }
}

/// Palette Update (MS-RDPBCGR 2.2.9.1.2)
///
/// Updates the color palette
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteUpdate {
    /// Palette entries (RGB triplets)
    pub entries: Vec<PaletteEntry>,
}

impl PaletteUpdate {
    /// Maximum number of palette entries (256)
    pub const MAX_ENTRIES: usize = 256;

    /// Create new palette update
    pub fn new(entries: Vec<PaletteEntry>) -> Self {
        Self { entries }
    }

    /// Minimum size (4 bytes)
    pub const MIN_SIZE: usize = 4;
}

impl Pdu for PaletteUpdate {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        // pad2Octets (2 bytes)
        buffer.write_u16::<LittleEndian>(0)?;
        // numberColors (2 bytes)
        buffer.write_u16::<LittleEndian>(self.entries.len() as u16)?;

        // Encode all palette entries
        for entry in &self.entries {
            entry.encode(buffer)?;
        }

        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let _pad = buffer.read_u16::<LittleEndian>()?;
        let number_colors = buffer.read_u16::<LittleEndian>()?;

        let mut entries = Vec::with_capacity(number_colors as usize);
        for _ in 0..number_colors {
            entries.push(PaletteEntry::decode(buffer)?);
        }

        Ok(Self { entries })
    }

    fn size(&self) -> usize {
        Self::MIN_SIZE + self.entries.len() * PaletteEntry::SIZE
    }
}

/// Palette Entry (MS-RDPBCGR 2.2.9.1.2)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteEntry {
    /// Red component
    pub red: u8,
    /// Green component
    pub green: u8,
    /// Blue component
    pub blue: u8,
}

impl PaletteEntry {
    /// Create new palette entry
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// Entry size (3 bytes)
    pub const SIZE: usize = 3;

    /// Encode palette entry
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u8(self.red)?;
        buffer.write_u8(self.green)?;
        buffer.write_u8(self.blue)?;
        Ok(())
    }

    /// Decode palette entry
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let red = buffer.read_u8()?;
        let green = buffer.read_u8()?;
        let blue = buffer.read_u8()?;

        Ok(Self { red, green, blue })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_update_type() {
        assert_eq!(UpdateType::Bitmap.as_u16(), 0x0001);
        assert_eq!(UpdateType::from_u16(0x0001), Some(UpdateType::Bitmap));
        assert_eq!(UpdateType::from_u16(0xFFFF), None);
    }

    #[test]
    fn test_update_pdu_synchronize() {
        let pdu = UpdatePdu::Synchronize;

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), 4); // 2 bytes type + 2 bytes padding

        let mut cursor = Cursor::new(buffer);
        let decoded = UpdatePdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded, pdu);
    }

    #[test]
    fn test_palette_entry() {
        let entry = PaletteEntry::new(255, 128, 64);

        let mut buffer = Vec::new();
        entry.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), PaletteEntry::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = PaletteEntry::decode(&mut cursor).unwrap();

        assert_eq!(decoded, entry);
        assert_eq!(decoded.red, 255);
        assert_eq!(decoded.green, 128);
        assert_eq!(decoded.blue, 64);
    }

    #[test]
    fn test_palette_update() {
        let entries = vec![
            PaletteEntry::new(255, 0, 0),
            PaletteEntry::new(0, 255, 0),
            PaletteEntry::new(0, 0, 255),
        ];
        let pdu = PaletteUpdate::new(entries);

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = PaletteUpdate::decode(&mut cursor).unwrap();

        assert_eq!(decoded.entries.len(), 3);
        assert_eq!(decoded.entries[0].red, 255);
        assert_eq!(decoded.entries[1].green, 255);
        assert_eq!(decoded.entries[2].blue, 255);
    }
}
