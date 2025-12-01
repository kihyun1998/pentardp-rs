use crate::pdu::{Pdu, PduError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Drawing Order Type (MS-RDPBCGR 2.2.2.2.1.1.2)
///
/// Primary drawing orders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OrderType {
    /// Destination blit (solid fill)
    DstBlt = 0x00,
    /// Pattern blit (pattern fill)
    PatBlt = 0x01,
    /// Screen blit (copy screen region)
    ScrBlt = 0x02,
    /// Draw line
    LineTo = 0x09,
    /// Opaque rectangle
    OpaqueRect = 0x0A,
    /// Save bitmap
    SaveBitmap = 0x0B,
    /// Memory blit (copy from bitmap cache)
    MemBlt = 0x0D,
    /// 3-way memory blit
    Mem3Blt = 0x0E,
    /// Multi-destination blit
    MultiDstBlt = 0x0F,
    /// Multi-pattern blit
    MultiPatBlt = 0x10,
    /// Multi-screen blit
    MultiScrBlt = 0x11,
    /// Multi-opaque rectangle
    MultiOpaqueRect = 0x12,
    /// Fast index (color table-based)
    FastIndex = 0x13,
    /// Polygon scanline (filled polygon)
    PolygonSC = 0x14,
    /// Polygon continue
    PolygonCB = 0x15,
    /// Polyline
    Polyline = 0x16,
    /// Fast glyph (monochrome text)
    FastGlyph = 0x18,
    /// Ellipse scanline
    EllipseSC = 0x19,
    /// Ellipse continue
    EllipseCB = 0x1A,
    /// Glyph index (text with cache)
    GlyphIndex = 0x1B,
}

impl OrderType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(OrderType::DstBlt),
            0x01 => Some(OrderType::PatBlt),
            0x02 => Some(OrderType::ScrBlt),
            0x09 => Some(OrderType::LineTo),
            0x0A => Some(OrderType::OpaqueRect),
            0x0B => Some(OrderType::SaveBitmap),
            0x0D => Some(OrderType::MemBlt),
            0x0E => Some(OrderType::Mem3Blt),
            0x0F => Some(OrderType::MultiDstBlt),
            0x10 => Some(OrderType::MultiPatBlt),
            0x11 => Some(OrderType::MultiScrBlt),
            0x12 => Some(OrderType::MultiOpaqueRect),
            0x13 => Some(OrderType::FastIndex),
            0x14 => Some(OrderType::PolygonSC),
            0x15 => Some(OrderType::PolygonCB),
            0x16 => Some(OrderType::Polyline),
            0x18 => Some(OrderType::FastGlyph),
            0x19 => Some(OrderType::EllipseSC),
            0x1A => Some(OrderType::EllipseCB),
            0x1B => Some(OrderType::GlyphIndex),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Drawing Order (MS-RDPBCGR 2.2.2.2.1.1.2)
///
/// Simplified implementation of primary drawing orders
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawingOrder {
    /// Destination blit order
    DstBlt(DstBltOrder),
    /// Pattern blit order
    PatBlt(PatBltOrder),
    /// Screen blit order
    ScrBlt(ScrBltOrder),
    /// Memory blit order
    MemBlt(MemBltOrder),
    /// Opaque rectangle order
    OpaqueRect(OpaqueRectOrder),
}

impl DrawingOrder {
    /// Get order type
    pub fn order_type(&self) -> OrderType {
        match self {
            DrawingOrder::DstBlt(_) => OrderType::DstBlt,
            DrawingOrder::PatBlt(_) => OrderType::PatBlt,
            DrawingOrder::ScrBlt(_) => OrderType::ScrBlt,
            DrawingOrder::MemBlt(_) => OrderType::MemBlt,
            DrawingOrder::OpaqueRect(_) => OrderType::OpaqueRect,
        }
    }

    /// Encode drawing order
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        // For simplicity, we encode just the order type
        // Full implementation would include control flags and field data
        buffer.write_u8(self.order_type().as_u8())?;

        match self {
            DrawingOrder::DstBlt(order) => order.encode(buffer)?,
            DrawingOrder::PatBlt(order) => order.encode(buffer)?,
            DrawingOrder::ScrBlt(order) => order.encode(buffer)?,
            DrawingOrder::MemBlt(order) => order.encode(buffer)?,
            DrawingOrder::OpaqueRect(order) => order.encode(buffer)?,
        }

        Ok(())
    }

    /// Decode drawing order (simplified)
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let order_type_value = buffer.read_u8()?;
        let order_type = OrderType::from_u8(order_type_value).ok_or_else(|| {
            PduError::ParseError(format!("Invalid order type: {:#x}", order_type_value))
        })?;

        match order_type {
            OrderType::DstBlt => Ok(DrawingOrder::DstBlt(DstBltOrder::decode(buffer)?)),
            OrderType::PatBlt => Ok(DrawingOrder::PatBlt(PatBltOrder::decode(buffer)?)),
            OrderType::ScrBlt => Ok(DrawingOrder::ScrBlt(ScrBltOrder::decode(buffer)?)),
            OrderType::MemBlt => Ok(DrawingOrder::MemBlt(MemBltOrder::decode(buffer)?)),
            OrderType::OpaqueRect => Ok(DrawingOrder::OpaqueRect(OpaqueRectOrder::decode(buffer)?)),
            _ => Err(PduError::ParseError(format!(
                "Unsupported order type: {:?}",
                order_type
            ))),
        }
    }

    /// Return size
    pub fn size(&self) -> usize {
        1 + match self {
            DrawingOrder::DstBlt(order) => order.size(),
            DrawingOrder::PatBlt(order) => order.size(),
            DrawingOrder::ScrBlt(order) => order.size(),
            DrawingOrder::MemBlt(order) => order.size(),
            DrawingOrder::OpaqueRect(order) => order.size(),
        }
    }
}

/// DstBlt Order (MS-RDPBCGR 2.2.2.2.1.1.2.1)
///
/// Destination blit - fills rectangle with solid color using ROP
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DstBltOrder {
    /// Left coordinate
    pub n_left_rect: i16,
    /// Top coordinate
    pub n_top_rect: i16,
    /// Width
    pub n_width: i16,
    /// Height
    pub n_height: i16,
    /// Raster operation (ROP3)
    pub b_rop: u8,
}

impl DstBltOrder {
    /// Create new DstBlt order
    pub fn new(x: i16, y: i16, width: i16, height: i16, rop: u8) -> Self {
        Self {
            n_left_rect: x,
            n_top_rect: y,
            n_width: width,
            n_height: height,
            b_rop: rop,
        }
    }

    /// Order data size (9 bytes)
    pub const SIZE: usize = 9;

    /// Encode
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_i16::<LittleEndian>(self.n_left_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_top_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_width)?;
        buffer.write_i16::<LittleEndian>(self.n_height)?;
        buffer.write_u8(self.b_rop)?;
        Ok(())
    }

    /// Decode
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        Ok(Self {
            n_left_rect: buffer.read_i16::<LittleEndian>()?,
            n_top_rect: buffer.read_i16::<LittleEndian>()?,
            n_width: buffer.read_i16::<LittleEndian>()?,
            n_height: buffer.read_i16::<LittleEndian>()?,
            b_rop: buffer.read_u8()?,
        })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

/// PatBlt Order (MS-RDPBCGR 2.2.2.2.1.1.2.2)
///
/// Pattern blit - fills rectangle with pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatBltOrder {
    /// Left coordinate
    pub n_left_rect: i16,
    /// Top coordinate
    pub n_top_rect: i16,
    /// Width
    pub n_width: i16,
    /// Height
    pub n_height: i16,
    /// Raster operation
    pub b_rop: u8,
    /// Background color (RGB)
    pub back_color: u32,
    /// Foreground color (RGB)
    pub fore_color: u32,
}

impl PatBltOrder {
    /// Create new PatBlt order
    pub fn new(
        x: i16,
        y: i16,
        width: i16,
        height: i16,
        rop: u8,
        back_color: u32,
        fore_color: u32,
    ) -> Self {
        Self {
            n_left_rect: x,
            n_top_rect: y,
            n_width: width,
            n_height: height,
            b_rop: rop,
            back_color,
            fore_color,
        }
    }

    /// Order data size (17 bytes)
    pub const SIZE: usize = 17;

    /// Encode
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_i16::<LittleEndian>(self.n_left_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_top_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_width)?;
        buffer.write_i16::<LittleEndian>(self.n_height)?;
        buffer.write_u8(self.b_rop)?;
        buffer.write_u32::<LittleEndian>(self.back_color)?;
        buffer.write_u32::<LittleEndian>(self.fore_color)?;
        Ok(())
    }

    /// Decode
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        Ok(Self {
            n_left_rect: buffer.read_i16::<LittleEndian>()?,
            n_top_rect: buffer.read_i16::<LittleEndian>()?,
            n_width: buffer.read_i16::<LittleEndian>()?,
            n_height: buffer.read_i16::<LittleEndian>()?,
            b_rop: buffer.read_u8()?,
            back_color: buffer.read_u32::<LittleEndian>()?,
            fore_color: buffer.read_u32::<LittleEndian>()?,
        })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

/// ScrBlt Order (MS-RDPBCGR 2.2.2.2.1.1.2.3)
///
/// Screen blit - copies region from screen
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrBltOrder {
    /// Destination left
    pub n_left_rect: i16,
    /// Destination top
    pub n_top_rect: i16,
    /// Width
    pub n_width: i16,
    /// Height
    pub n_height: i16,
    /// Raster operation
    pub b_rop: u8,
    /// Source X coordinate
    pub n_x_src: i16,
    /// Source Y coordinate
    pub n_y_src: i16,
}

impl ScrBltOrder {
    /// Create new ScrBlt order
    pub fn new(
        dest_x: i16,
        dest_y: i16,
        width: i16,
        height: i16,
        rop: u8,
        src_x: i16,
        src_y: i16,
    ) -> Self {
        Self {
            n_left_rect: dest_x,
            n_top_rect: dest_y,
            n_width: width,
            n_height: height,
            b_rop: rop,
            n_x_src: src_x,
            n_y_src: src_y,
        }
    }

    /// Order data size (13 bytes)
    pub const SIZE: usize = 13;

    /// Encode
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_i16::<LittleEndian>(self.n_left_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_top_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_width)?;
        buffer.write_i16::<LittleEndian>(self.n_height)?;
        buffer.write_u8(self.b_rop)?;
        buffer.write_i16::<LittleEndian>(self.n_x_src)?;
        buffer.write_i16::<LittleEndian>(self.n_y_src)?;
        Ok(())
    }

    /// Decode
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        Ok(Self {
            n_left_rect: buffer.read_i16::<LittleEndian>()?,
            n_top_rect: buffer.read_i16::<LittleEndian>()?,
            n_width: buffer.read_i16::<LittleEndian>()?,
            n_height: buffer.read_i16::<LittleEndian>()?,
            b_rop: buffer.read_u8()?,
            n_x_src: buffer.read_i16::<LittleEndian>()?,
            n_y_src: buffer.read_i16::<LittleEndian>()?,
        })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

/// MemBlt Order (MS-RDPBCGR 2.2.2.2.1.1.2.9)
///
/// Memory blit - copies from bitmap cache
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemBltOrder {
    /// Cache ID
    pub cache_id: u16,
    /// Destination left
    pub n_left_rect: i16,
    /// Destination top
    pub n_top_rect: i16,
    /// Width
    pub n_width: i16,
    /// Height
    pub n_height: i16,
    /// Raster operation
    pub b_rop: u8,
    /// Source X coordinate
    pub n_x_src: i16,
    /// Source Y coordinate
    pub n_y_src: i16,
    /// Cache index
    pub cache_index: u16,
}

impl MemBltOrder {
    /// Create new MemBlt order
    pub fn new(
        cache_id: u16,
        dest_x: i16,
        dest_y: i16,
        width: i16,
        height: i16,
        rop: u8,
        src_x: i16,
        src_y: i16,
        cache_index: u16,
    ) -> Self {
        Self {
            cache_id,
            n_left_rect: dest_x,
            n_top_rect: dest_y,
            n_width: width,
            n_height: height,
            b_rop: rop,
            n_x_src: src_x,
            n_y_src: src_y,
            cache_index,
        }
    }

    /// Order data size (17 bytes)
    pub const SIZE: usize = 17;

    /// Encode
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.cache_id)?;
        buffer.write_i16::<LittleEndian>(self.n_left_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_top_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_width)?;
        buffer.write_i16::<LittleEndian>(self.n_height)?;
        buffer.write_u8(self.b_rop)?;
        buffer.write_i16::<LittleEndian>(self.n_x_src)?;
        buffer.write_i16::<LittleEndian>(self.n_y_src)?;
        buffer.write_u16::<LittleEndian>(self.cache_index)?;
        Ok(())
    }

    /// Decode
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        Ok(Self {
            cache_id: buffer.read_u16::<LittleEndian>()?,
            n_left_rect: buffer.read_i16::<LittleEndian>()?,
            n_top_rect: buffer.read_i16::<LittleEndian>()?,
            n_width: buffer.read_i16::<LittleEndian>()?,
            n_height: buffer.read_i16::<LittleEndian>()?,
            b_rop: buffer.read_u8()?,
            n_x_src: buffer.read_i16::<LittleEndian>()?,
            n_y_src: buffer.read_i16::<LittleEndian>()?,
            cache_index: buffer.read_u16::<LittleEndian>()?,
        })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

/// Opaque Rectangle Order (MS-RDPBCGR 2.2.2.2.1.1.2.5)
///
/// Draws filled rectangle with solid color
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpaqueRectOrder {
    /// Left coordinate
    pub n_left_rect: i16,
    /// Top coordinate
    pub n_top_rect: i16,
    /// Width
    pub n_width: i16,
    /// Height
    pub n_height: i16,
    /// Color (RGB)
    pub color: u32,
}

impl OpaqueRectOrder {
    /// Create new opaque rectangle order
    pub fn new(x: i16, y: i16, width: i16, height: i16, color: u32) -> Self {
        Self {
            n_left_rect: x,
            n_top_rect: y,
            n_width: width,
            n_height: height,
            color,
        }
    }

    /// Order data size (12 bytes)
    pub const SIZE: usize = 12;

    /// Encode
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_i16::<LittleEndian>(self.n_left_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_top_rect)?;
        buffer.write_i16::<LittleEndian>(self.n_width)?;
        buffer.write_i16::<LittleEndian>(self.n_height)?;
        buffer.write_u32::<LittleEndian>(self.color)?;
        Ok(())
    }

    /// Decode
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        Ok(Self {
            n_left_rect: buffer.read_i16::<LittleEndian>()?,
            n_top_rect: buffer.read_i16::<LittleEndian>()?,
            n_width: buffer.read_i16::<LittleEndian>()?,
            n_height: buffer.read_i16::<LittleEndian>()?,
            color: buffer.read_u32::<LittleEndian>()?,
        })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

/// Orders Update (MS-RDPBCGR 2.2.9.1.1.3.1.1)
///
/// Container for drawing orders
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrdersUpdate {
    /// Number of drawing orders
    pub number_orders: u16,
    /// Drawing orders
    pub orders: Vec<DrawingOrder>,
}

impl OrdersUpdate {
    /// Create new orders update
    pub fn new(orders: Vec<DrawingOrder>) -> Self {
        let number_orders = orders.len() as u16;
        Self {
            number_orders,
            orders,
        }
    }

    /// Create update with single order
    pub fn single(order: DrawingOrder) -> Self {
        Self::new(vec![order])
    }

    /// Minimum size (4 bytes: 2 for pad + 2 for number_orders)
    pub const MIN_SIZE: usize = 4;
}

impl Pdu for OrdersUpdate {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        // pad2Octets (2 bytes)
        buffer.write_u16::<LittleEndian>(0)?;
        // numberOrders (2 bytes)
        buffer.write_u16::<LittleEndian>(self.number_orders)?;

        // Encode all orders
        for order in &self.orders {
            order.encode(buffer)?;
        }

        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let _pad = buffer.read_u16::<LittleEndian>()?;
        let number_orders = buffer.read_u16::<LittleEndian>()?;

        let mut orders = Vec::with_capacity(number_orders as usize);
        for _ in 0..number_orders {
            orders.push(DrawingOrder::decode(buffer)?);
        }

        Ok(Self {
            number_orders,
            orders,
        })
    }

    fn size(&self) -> usize {
        Self::MIN_SIZE + self.orders.iter().map(|o| o.size()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_order_type() {
        assert_eq!(OrderType::DstBlt.as_u8(), 0x00);
        assert_eq!(OrderType::from_u8(0x00), Some(OrderType::DstBlt));
        assert_eq!(OrderType::from_u8(0xFF), None);
    }

    #[test]
    fn test_dstblt_order_encode_decode() {
        let order = DstBltOrder::new(10, 20, 100, 50, 0xCC);

        let mut buffer = Vec::new();
        order.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), DstBltOrder::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = DstBltOrder::decode(&mut cursor).unwrap();

        assert_eq!(decoded, order);
        assert_eq!(decoded.n_left_rect, 10);
        assert_eq!(decoded.n_top_rect, 20);
        assert_eq!(decoded.n_width, 100);
        assert_eq!(decoded.n_height, 50);
        assert_eq!(decoded.b_rop, 0xCC);
    }

    #[test]
    fn test_patblt_order_encode_decode() {
        let order = PatBltOrder::new(5, 5, 50, 50, 0xF0, 0xFF0000, 0x00FF00);

        let mut buffer = Vec::new();
        order.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = PatBltOrder::decode(&mut cursor).unwrap();

        assert_eq!(decoded, order);
        assert_eq!(decoded.back_color, 0xFF0000);
        assert_eq!(decoded.fore_color, 0x00FF00);
    }

    #[test]
    fn test_scrblt_order_encode_decode() {
        let order = ScrBltOrder::new(100, 100, 64, 64, 0xCC, 50, 50);

        let mut buffer = Vec::new();
        order.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ScrBltOrder::decode(&mut cursor).unwrap();

        assert_eq!(decoded, order);
        assert_eq!(decoded.n_x_src, 50);
        assert_eq!(decoded.n_y_src, 50);
    }

    #[test]
    fn test_memblt_order_encode_decode() {
        let order = MemBltOrder::new(0, 10, 10, 32, 32, 0xCC, 0, 0, 5);

        let mut buffer = Vec::new();
        order.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = MemBltOrder::decode(&mut cursor).unwrap();

        assert_eq!(decoded, order);
        assert_eq!(decoded.cache_id, 0);
        assert_eq!(decoded.cache_index, 5);
    }

    #[test]
    fn test_opaque_rect_order_encode_decode() {
        let order = OpaqueRectOrder::new(0, 0, 800, 600, 0x0000FF);

        let mut buffer = Vec::new();
        order.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = OpaqueRectOrder::decode(&mut cursor).unwrap();

        assert_eq!(decoded, order);
        assert_eq!(decoded.color, 0x0000FF);
    }

    #[test]
    fn test_drawing_order_encode_decode() {
        let order = DrawingOrder::DstBlt(DstBltOrder::new(5, 5, 10, 10, 0xCC));

        let mut buffer = Vec::new();
        order.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = DrawingOrder::decode(&mut cursor).unwrap();

        assert_eq!(decoded, order);
    }

    #[test]
    fn test_orders_update_single() {
        let order = DrawingOrder::OpaqueRect(OpaqueRectOrder::new(0, 0, 100, 100, 0xFF0000));
        let update = OrdersUpdate::single(order);

        let mut buffer = Vec::new();
        update.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = OrdersUpdate::decode(&mut cursor).unwrap();

        assert_eq!(decoded.number_orders, 1);
        assert_eq!(decoded.orders.len(), 1);
    }

    #[test]
    fn test_orders_update_multiple() {
        let orders = vec![
            DrawingOrder::DstBlt(DstBltOrder::new(0, 0, 10, 10, 0xCC)),
            DrawingOrder::PatBlt(PatBltOrder::new(10, 10, 20, 20, 0xF0, 0, 0xFFFFFF)),
            DrawingOrder::ScrBlt(ScrBltOrder::new(30, 30, 15, 15, 0xCC, 0, 0)),
        ];
        let update = OrdersUpdate::new(orders);

        let mut buffer = Vec::new();
        update.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = OrdersUpdate::decode(&mut cursor).unwrap();

        assert_eq!(decoded.number_orders, 3);
        assert_eq!(decoded.orders.len(), 3);
    }
}
