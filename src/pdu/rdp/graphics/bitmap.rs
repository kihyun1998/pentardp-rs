use crate::pdu::{Pdu, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Bitmap Flags (MS-RDPBCGR 2.2.9.1.1.3.1.2.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitmapFlags(u16);

impl BitmapFlags {
    /// Bitmap data is compressed
    pub const COMPRESSED: u16 = 0x0001;
    /// No bitmap compression header (used with COMPRESSED)
    pub const NO_BITMAP_COMPRESSION_HDR: u16 = 0x0400;

    /// Create new bitmap flags
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    /// Create flags for uncompressed bitmap
    pub fn uncompressed() -> Self {
        Self(0)
    }

    /// Create flags for compressed bitmap
    pub fn compressed() -> Self {
        Self(Self::COMPRESSED)
    }

    /// Create flags for compressed bitmap without header
    pub fn compressed_no_header() -> Self {
        Self(Self::COMPRESSED | Self::NO_BITMAP_COMPRESSION_HDR)
    }

    /// Check if bitmap is compressed
    pub fn is_compressed(&self) -> bool {
        self.0 & Self::COMPRESSED != 0
    }

    /// Check if compression header is absent
    pub fn no_compression_header(&self) -> bool {
        self.0 & Self::NO_BITMAP_COMPRESSION_HDR != 0
    }

    /// Get raw value
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Bitmap Data (MS-RDPBCGR 2.2.9.1.1.3.1.2.2)
///
/// Represents a single bitmap rectangle update
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapData {
    /// Destination left coordinate
    pub dest_left: u16,
    /// Destination top coordinate
    pub dest_top: u16,
    /// Destination right coordinate
    pub dest_right: u16,
    /// Destination bottom coordinate
    pub dest_bottom: u16,
    /// Bitmap width in pixels
    pub width: u16,
    /// Bitmap height in pixels
    pub height: u16,
    /// Bits per pixel (8, 15, 16, 24, 32)
    pub bits_per_pixel: u16,
    /// Bitmap flags
    pub flags: BitmapFlags,
    /// Length of bitmap data in bytes
    pub bitmap_length: u16,
    /// Bitmap pixel data (may be compressed)
    pub bitmap_data: Vec<u8>,
}

impl BitmapData {
    /// Create new bitmap data
    pub fn new(
        dest_left: u16,
        dest_top: u16,
        dest_right: u16,
        dest_bottom: u16,
        width: u16,
        height: u16,
        bits_per_pixel: u16,
        flags: BitmapFlags,
        bitmap_data: Vec<u8>,
    ) -> Self {
        let bitmap_length = bitmap_data.len() as u16;
        Self {
            dest_left,
            dest_top,
            dest_right,
            dest_bottom,
            width,
            height,
            bits_per_pixel,
            flags,
            bitmap_length,
            bitmap_data,
        }
    }

    /// Create uncompressed bitmap data
    pub fn uncompressed(
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        bits_per_pixel: u16,
        bitmap_data: Vec<u8>,
    ) -> Self {
        Self::new(
            x,
            y,
            x + width - 1,
            y + height - 1,
            width,
            height,
            bits_per_pixel,
            BitmapFlags::uncompressed(),
            bitmap_data,
        )
    }

    /// Minimum header size (18 bytes, not including bitmap data)
    pub const HEADER_SIZE: usize = 18;

    /// Encode bitmap data
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.dest_left)?;
        buffer.write_u16::<LittleEndian>(self.dest_top)?;
        buffer.write_u16::<LittleEndian>(self.dest_right)?;
        buffer.write_u16::<LittleEndian>(self.dest_bottom)?;
        buffer.write_u16::<LittleEndian>(self.width)?;
        buffer.write_u16::<LittleEndian>(self.height)?;
        buffer.write_u16::<LittleEndian>(self.bits_per_pixel)?;
        buffer.write_u16::<LittleEndian>(self.flags.as_u16())?;
        buffer.write_u16::<LittleEndian>(self.bitmap_length)?;

        // Bitmap data
        buffer.write_all(&self.bitmap_data)?;

        Ok(())
    }

    /// Decode bitmap data
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let dest_left = buffer.read_u16::<LittleEndian>()?;
        let dest_top = buffer.read_u16::<LittleEndian>()?;
        let dest_right = buffer.read_u16::<LittleEndian>()?;
        let dest_bottom = buffer.read_u16::<LittleEndian>()?;
        let width = buffer.read_u16::<LittleEndian>()?;
        let height = buffer.read_u16::<LittleEndian>()?;
        let bits_per_pixel = buffer.read_u16::<LittleEndian>()?;
        let flags = BitmapFlags::new(buffer.read_u16::<LittleEndian>()?);
        let bitmap_length = buffer.read_u16::<LittleEndian>()?;

        // Read bitmap data
        let mut bitmap_data = vec![0u8; bitmap_length as usize];
        buffer.read_exact(&mut bitmap_data)?;

        Ok(Self {
            dest_left,
            dest_top,
            dest_right,
            dest_bottom,
            width,
            height,
            bits_per_pixel,
            flags,
            bitmap_length,
            bitmap_data,
        })
    }

    /// Return total size
    pub fn size(&self) -> usize {
        Self::HEADER_SIZE + self.bitmap_data.len()
    }
}

/// Bitmap Update (MS-RDPBCGR 2.2.9.1.1.3.1.2)
///
/// Container for multiple bitmap rectangles
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapUpdate {
    /// Number of bitmap rectangles
    pub number_rectangles: u16,
    /// Bitmap rectangles
    pub rectangles: Vec<BitmapData>,
}

impl BitmapUpdate {
    /// Create new bitmap update
    pub fn new(rectangles: Vec<BitmapData>) -> Self {
        let number_rectangles = rectangles.len() as u16;
        Self {
            number_rectangles,
            rectangles,
        }
    }

    /// Create update with single rectangle
    pub fn single(bitmap: BitmapData) -> Self {
        Self::new(vec![bitmap])
    }

    /// Minimum size (2 bytes for number_rectangles)
    pub const MIN_SIZE: usize = 2;
}

impl Pdu for BitmapUpdate {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        // numberRectangles (2 bytes)
        buffer.write_u16::<LittleEndian>(self.number_rectangles)?;

        // Encode all rectangles
        for rect in &self.rectangles {
            rect.encode(buffer)?;
        }

        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let number_rectangles = buffer.read_u16::<LittleEndian>()?;

        let mut rectangles = Vec::with_capacity(number_rectangles as usize);
        for _ in 0..number_rectangles {
            rectangles.push(BitmapData::decode(buffer)?);
        }

        Ok(Self {
            number_rectangles,
            rectangles,
        })
    }

    fn size(&self) -> usize {
        Self::MIN_SIZE + self.rectangles.iter().map(|r| r.size()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_bitmap_flags() {
        let flags = BitmapFlags::uncompressed();
        assert!(!flags.is_compressed());
        assert_eq!(flags.as_u16(), 0);

        let flags = BitmapFlags::compressed();
        assert!(flags.is_compressed());
        assert!(!flags.no_compression_header());

        let flags = BitmapFlags::compressed_no_header();
        assert!(flags.is_compressed());
        assert!(flags.no_compression_header());
    }

    #[test]
    fn test_bitmap_data_encode_decode() {
        let pixel_data = vec![0xFF, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0xFF];
        let bitmap = BitmapData::uncompressed(10, 20, 3, 1, 24, pixel_data.clone());

        let mut buffer = Vec::new();
        bitmap.encode(&mut buffer).unwrap();

        let expected_size = BitmapData::HEADER_SIZE + pixel_data.len();
        assert_eq!(buffer.len(), expected_size);

        let mut cursor = Cursor::new(buffer);
        let decoded = BitmapData::decode(&mut cursor).unwrap();

        assert_eq!(decoded, bitmap);
        assert_eq!(decoded.dest_left, 10);
        assert_eq!(decoded.dest_top, 20);
        assert_eq!(decoded.width, 3);
        assert_eq!(decoded.height, 1);
        assert_eq!(decoded.bits_per_pixel, 24);
        assert_eq!(decoded.bitmap_data, pixel_data);
    }

    #[test]
    fn test_bitmap_data_coordinates() {
        let bitmap = BitmapData::uncompressed(100, 200, 64, 48, 16, vec![0; 64 * 48 * 2]);

        assert_eq!(bitmap.dest_left, 100);
        assert_eq!(bitmap.dest_top, 200);
        assert_eq!(bitmap.dest_right, 100 + 64 - 1);
        assert_eq!(bitmap.dest_bottom, 200 + 48 - 1);
    }

    #[test]
    fn test_bitmap_update_single() {
        let pixel_data = vec![0xAA; 16];
        let bitmap = BitmapData::uncompressed(0, 0, 4, 4, 8, pixel_data);
        let update = BitmapUpdate::single(bitmap);

        let mut buffer = Vec::new();
        update.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = BitmapUpdate::decode(&mut cursor).unwrap();

        assert_eq!(decoded.number_rectangles, 1);
        assert_eq!(decoded.rectangles.len(), 1);
        assert_eq!(decoded.rectangles[0].bitmap_data.len(), 16);
    }

    #[test]
    fn test_bitmap_update_multiple() {
        let bitmaps = vec![
            BitmapData::uncompressed(0, 0, 8, 8, 8, vec![0xFF; 64]),
            BitmapData::uncompressed(8, 0, 8, 8, 8, vec![0x00; 64]),
            BitmapData::uncompressed(0, 8, 8, 8, 8, vec![0xAA; 64]),
        ];
        let update = BitmapUpdate::new(bitmaps);

        let mut buffer = Vec::new();
        update.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = BitmapUpdate::decode(&mut cursor).unwrap();

        assert_eq!(decoded.number_rectangles, 3);
        assert_eq!(decoded.rectangles.len(), 3);
        assert_eq!(decoded.rectangles[0].bitmap_data[0], 0xFF);
        assert_eq!(decoded.rectangles[1].bitmap_data[0], 0x00);
        assert_eq!(decoded.rectangles[2].bitmap_data[0], 0xAA);
    }

    #[test]
    fn test_bitmap_data_compressed_flag() {
        let bitmap = BitmapData::new(
            0,
            0,
            15,
            15,
            16,
            16,
            8,
            BitmapFlags::compressed(),
            vec![0x12, 0x34, 0x56],
        );

        assert!(bitmap.flags.is_compressed());
        assert_eq!(bitmap.bitmap_length, 3);
    }

    #[test]
    fn test_bitmap_update_size() {
        let bitmaps = vec![
            BitmapData::uncompressed(0, 0, 2, 2, 8, vec![0; 4]),
            BitmapData::uncompressed(2, 2, 2, 2, 8, vec![0; 4]),
        ];
        let update = BitmapUpdate::new(bitmaps);

        let expected_size = BitmapUpdate::MIN_SIZE + 2 * (BitmapData::HEADER_SIZE + 4);
        assert_eq!(update.size(), expected_size);
    }
}
