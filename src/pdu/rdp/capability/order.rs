use crate::pdu::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use super::sets::{CapabilitySetHeader, CapabilitySetType};

/// Order Capability Set (MS-RDPBCGR 2.2.7.1.3)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderCapability {
    /// Terminal descriptor (16 bytes)
    pub terminal_descriptor: [u8; 16],
    /// Desktop save X granularity
    pub desktop_save_x_granularity: u16,
    /// Desktop save Y granularity
    pub desktop_save_y_granularity: u16,
    /// Maximum order level (1)
    pub maximum_order_level: u16,
    /// Number of fonts (0)
    pub number_fonts: u16,
    /// Order flags
    pub order_flags: u16,
    /// Order support array (32 bytes)
    pub order_support: [u8; 32],
    /// Text flags
    pub text_flags: u16,
    /// Order support extended flags
    pub order_support_ex_flags: u16,
    /// Desktop save size (0 - not used)
    pub desktop_save_size: u32,
    /// Text ANSI code page
    pub text_ansi_code_page: u16,
}

impl OrderCapability {
    /// Create new Order Capability with default support
    pub fn new() -> Self {
        let mut order_support = [0u8; 32];

        // Enable common drawing orders
        order_support[0] = 1;  // TS_NEG_DSTBLT_INDEX
        order_support[1] = 1;  // TS_NEG_PATBLT_INDEX
        order_support[2] = 1;  // TS_NEG_SCRBLT_INDEX
        order_support[3] = 1;  // TS_NEG_MEMBLT_INDEX
        order_support[4] = 1;  // TS_NEG_MEM3BLT_INDEX
        order_support[8] = 1;  // TS_NEG_LINETO_INDEX
        order_support[9] = 1;  // TS_NEG_MULTI_DRAWNINEGRID_INDEX
        order_support[15] = 1; // TS_NEG_MULTIDSTBLT_INDEX
        order_support[16] = 1; // TS_NEG_MULTIPATBLT_INDEX
        order_support[17] = 1; // TS_NEG_MULTISCRBLT_INDEX
        order_support[18] = 1; // TS_NEG_MULTIOPAQUERECT_INDEX
        order_support[22] = 1; // TS_NEG_POLYLINE_INDEX
        order_support[25] = 1; // TS_NEG_ELLIPSE_SC_INDEX
        order_support[27] = 1; // TS_NEG_INDEX_INDEX

        Self {
            terminal_descriptor: [0; 16],
            desktop_save_x_granularity: 1,
            desktop_save_y_granularity: 20,
            maximum_order_level: 1,
            number_fonts: 0,
            order_flags: 0x00AA, // NEGOTIATE_ORDER_SUPPORT | ZERO_BOUNDS_DELTAS_SUPPORT | COLOR_INDEX_SUPPORT
            order_support,
            text_flags: 0x06A1, // Various text support flags
            order_support_ex_flags: 0,
            desktop_save_size: 0,
            text_ansi_code_page: 1252, // Western European (Windows)
        }
    }

    /// Data size (excluding header)
    pub const DATA_SIZE: usize = 84;

    /// Encode capability set (with header)
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let header = CapabilitySetHeader::new(
            CapabilitySetType::Order,
            (CapabilitySetHeader::SIZE + Self::DATA_SIZE) as u16,
        );
        header.encode(buffer)?;

        buffer.write_all(&self.terminal_descriptor)?;
        buffer.write_u32::<LittleEndian>(0)?; // padding
        buffer.write_u16::<LittleEndian>(self.desktop_save_x_granularity)?;
        buffer.write_u16::<LittleEndian>(self.desktop_save_y_granularity)?;
        buffer.write_u16::<LittleEndian>(0)?; // padding
        buffer.write_u16::<LittleEndian>(self.maximum_order_level)?;
        buffer.write_u16::<LittleEndian>(self.number_fonts)?;
        buffer.write_u16::<LittleEndian>(self.order_flags)?;
        buffer.write_all(&self.order_support)?;
        buffer.write_u16::<LittleEndian>(self.text_flags)?;
        buffer.write_u16::<LittleEndian>(self.order_support_ex_flags)?;
        buffer.write_u32::<LittleEndian>(0)?; // padding
        buffer.write_u32::<LittleEndian>(self.desktop_save_size)?;
        buffer.write_u16::<LittleEndian>(0)?; // padding
        buffer.write_u16::<LittleEndian>(0)?; // padding
        buffer.write_u16::<LittleEndian>(self.text_ansi_code_page)?;
        buffer.write_u16::<LittleEndian>(0)?; // padding

        Ok(())
    }

    /// Decode capability data (without header)
    pub fn decode_data(buffer: &mut dyn Read, _data_len: usize) -> Result<Self> {
        let mut terminal_descriptor = [0u8; 16];
        buffer.read_exact(&mut terminal_descriptor)?;

        let _padding1 = buffer.read_u32::<LittleEndian>()?;
        let desktop_save_x_granularity = buffer.read_u16::<LittleEndian>()?;
        let desktop_save_y_granularity = buffer.read_u16::<LittleEndian>()?;
        let _padding2 = buffer.read_u16::<LittleEndian>()?;
        let maximum_order_level = buffer.read_u16::<LittleEndian>()?;
        let number_fonts = buffer.read_u16::<LittleEndian>()?;
        let order_flags = buffer.read_u16::<LittleEndian>()?;

        let mut order_support = [0u8; 32];
        buffer.read_exact(&mut order_support)?;

        let text_flags = buffer.read_u16::<LittleEndian>()?;
        let order_support_ex_flags = buffer.read_u16::<LittleEndian>()?;
        let _padding3 = buffer.read_u32::<LittleEndian>()?;
        let desktop_save_size = buffer.read_u32::<LittleEndian>()?;
        let _padding4 = buffer.read_u16::<LittleEndian>()?;
        let _padding5 = buffer.read_u16::<LittleEndian>()?;
        let text_ansi_code_page = buffer.read_u16::<LittleEndian>()?;
        let _padding6 = buffer.read_u16::<LittleEndian>()?;

        Ok(Self {
            terminal_descriptor,
            desktop_save_x_granularity,
            desktop_save_y_granularity,
            maximum_order_level,
            number_fonts,
            order_flags,
            order_support,
            text_flags,
            order_support_ex_flags,
            desktop_save_size,
            text_ansi_code_page,
        })
    }

    /// Get size (including header)
    pub fn size(&self) -> usize {
        CapabilitySetHeader::SIZE + Self::DATA_SIZE
    }
}

impl Default for OrderCapability {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_order_capability() {
        let cap = OrderCapability::new();

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), cap.size());

        let mut cursor = Cursor::new(&buffer[CapabilitySetHeader::SIZE..]);
        let decoded = OrderCapability::decode_data(&mut cursor, OrderCapability::DATA_SIZE).unwrap();

        assert_eq!(decoded.maximum_order_level, 1);
        assert_eq!(decoded.text_ansi_code_page, 1252);
    }

    #[test]
    fn test_order_capability_roundtrip() {
        let cap = OrderCapability::new();

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(&buffer[CapabilitySetHeader::SIZE..]);
        let decoded = OrderCapability::decode_data(&mut cursor, OrderCapability::DATA_SIZE).unwrap();

        assert_eq!(cap, decoded);
    }

    #[test]
    fn test_order_support_array() {
        let cap = OrderCapability::new();

        // Check that some common orders are supported
        assert_eq!(cap.order_support[0], 1); // DSTBLT
        assert_eq!(cap.order_support[1], 1); // PATBLT
        assert_eq!(cap.order_support[2], 1); // SCRBLT
    }
}
