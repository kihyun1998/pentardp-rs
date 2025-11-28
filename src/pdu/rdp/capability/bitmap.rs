use crate::pdu::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use super::sets::{CapabilitySetHeader, CapabilitySetType};

/// Bitmap Capability Set (MS-RDPBCGR 2.2.7.1.2)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapCapability {
    /// Preferred bits per pixel (8, 15, 16, 24, 32)
    pub preferred_bits_per_pixel: u16,
    /// Receive 1 bits per pixel (1 = supported)
    pub receive1_bits_per_pixel: u16,
    /// Receive 4 bits per pixel (1 = supported)
    pub receive4_bits_per_pixel: u16,
    /// Receive 8 bits per pixel (1 = supported)
    pub receive8_bits_per_pixel: u16,
    /// Desktop width in pixels
    pub desktop_width: u16,
    /// Desktop height in pixels
    pub desktop_height: u16,
    /// Desktop resize flag (1 = supported)
    pub desktop_resize_flag: u16,
    /// Bitmap compression flag (1 = supported)
    pub bitmap_compression_flag: u16,
    /// High color flags
    pub high_color_flags: u8,
    /// Drawing flags
    pub drawing_flags: u8,
    /// Multiple rectangle support (1 = supported)
    pub multiple_rectangle_support: u16,
}

impl BitmapCapability {
    /// Create new Bitmap Capability
    pub fn new(width: u16, height: u16, bits_per_pixel: u16) -> Self {
        Self {
            preferred_bits_per_pixel: bits_per_pixel,
            receive1_bits_per_pixel: 1,
            receive4_bits_per_pixel: 1,
            receive8_bits_per_pixel: 1,
            desktop_width: width,
            desktop_height: height,
            desktop_resize_flag: 1, // Support desktop resize
            bitmap_compression_flag: 1,
            high_color_flags: 0,
            drawing_flags: 0x1B, // DRAW_ALLOW_DYNAMIC_COLOR_FIDELITY | DRAW_ALLOW_COLOR_SUBSAMPLING | DRAW_ALLOW_SKIP_ALPHA
            multiple_rectangle_support: 1,
        }
    }

    /// Default 1920x1080 32bpp
    pub fn default_1080p() -> Self {
        Self::new(1920, 1080, 32)
    }

    /// Data size (excluding header)
    pub const DATA_SIZE: usize = 24;

    /// Encode capability set (with header)
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let header = CapabilitySetHeader::new(
            CapabilitySetType::Bitmap,
            (CapabilitySetHeader::SIZE + Self::DATA_SIZE) as u16,
        );
        header.encode(buffer)?;

        buffer.write_u16::<LittleEndian>(self.preferred_bits_per_pixel)?;
        buffer.write_u16::<LittleEndian>(self.receive1_bits_per_pixel)?;
        buffer.write_u16::<LittleEndian>(self.receive4_bits_per_pixel)?;
        buffer.write_u16::<LittleEndian>(self.receive8_bits_per_pixel)?;
        buffer.write_u16::<LittleEndian>(self.desktop_width)?;
        buffer.write_u16::<LittleEndian>(self.desktop_height)?;
        buffer.write_u16::<LittleEndian>(0)?; // padding
        buffer.write_u16::<LittleEndian>(self.desktop_resize_flag)?;
        buffer.write_u16::<LittleEndian>(self.bitmap_compression_flag)?;
        buffer.write_u8(self.high_color_flags)?;
        buffer.write_u8(self.drawing_flags)?;
        buffer.write_u16::<LittleEndian>(self.multiple_rectangle_support)?;
        buffer.write_u16::<LittleEndian>(0)?; // padding

        Ok(())
    }

    /// Decode capability data (without header)
    pub fn decode_data(buffer: &mut dyn Read, _data_len: usize) -> Result<Self> {
        let preferred_bits_per_pixel = buffer.read_u16::<LittleEndian>()?;
        let receive1_bits_per_pixel = buffer.read_u16::<LittleEndian>()?;
        let receive4_bits_per_pixel = buffer.read_u16::<LittleEndian>()?;
        let receive8_bits_per_pixel = buffer.read_u16::<LittleEndian>()?;
        let desktop_width = buffer.read_u16::<LittleEndian>()?;
        let desktop_height = buffer.read_u16::<LittleEndian>()?;
        let _padding1 = buffer.read_u16::<LittleEndian>()?;
        let desktop_resize_flag = buffer.read_u16::<LittleEndian>()?;
        let bitmap_compression_flag = buffer.read_u16::<LittleEndian>()?;
        let high_color_flags = buffer.read_u8()?;
        let drawing_flags = buffer.read_u8()?;
        let multiple_rectangle_support = buffer.read_u16::<LittleEndian>()?;
        let _padding2 = buffer.read_u16::<LittleEndian>()?;

        Ok(Self {
            preferred_bits_per_pixel,
            receive1_bits_per_pixel,
            receive4_bits_per_pixel,
            receive8_bits_per_pixel,
            desktop_width,
            desktop_height,
            desktop_resize_flag,
            bitmap_compression_flag,
            high_color_flags,
            drawing_flags,
            multiple_rectangle_support,
        })
    }

    /// Get size (including header)
    pub fn size(&self) -> usize {
        CapabilitySetHeader::SIZE + Self::DATA_SIZE
    }
}

impl Default for BitmapCapability {
    fn default() -> Self {
        Self::default_1080p()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_bitmap_capability() {
        let cap = BitmapCapability::new(1920, 1080, 32);

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), cap.size());

        let mut cursor = Cursor::new(&buffer[CapabilitySetHeader::SIZE..]);
        let decoded = BitmapCapability::decode_data(&mut cursor, BitmapCapability::DATA_SIZE).unwrap();

        assert_eq!(decoded.desktop_width, 1920);
        assert_eq!(decoded.desktop_height, 1080);
        assert_eq!(decoded.preferred_bits_per_pixel, 32);
    }

    #[test]
    fn test_bitmap_capability_roundtrip() {
        let cap = BitmapCapability::default_1080p();

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(&buffer[CapabilitySetHeader::SIZE..]);
        let decoded = BitmapCapability::decode_data(&mut cursor, BitmapCapability::DATA_SIZE).unwrap();

        assert_eq!(cap, decoded);
    }
}
