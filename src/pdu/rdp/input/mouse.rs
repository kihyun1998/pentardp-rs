use crate::pdu::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Mouse Event Flags (MS-RDPBCGR 2.2.8.1.1.3.1.1.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseFlags(u16);

impl MouseFlags {
    /// Mouse move event
    pub const MOVE: u16 = 0x0800;
    /// Left button pressed
    pub const DOWN: u16 = 0x8000;
    /// Left button (same as BUTTON1)
    pub const BUTTON1: u16 = 0x1000;
    /// Right button
    pub const BUTTON2: u16 = 0x2000;
    /// Middle button
    pub const BUTTON3: u16 = 0x4000;
    /// Vertical wheel rotation
    pub const WHEEL: u16 = 0x0200;
    /// Horizontal wheel rotation
    pub const HWHEEL: u16 = 0x0400;
    /// Wheel rotation is negative (down/left)
    pub const WHEEL_NEGATIVE: u16 = 0x0100;

    /// Create new mouse flags
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    /// Create flags for mouse move
    pub fn move_event() -> Self {
        Self(Self::MOVE)
    }

    /// Create flags for left button down
    pub fn left_button_down() -> Self {
        Self(Self::DOWN | Self::BUTTON1)
    }

    /// Create flags for left button up
    pub fn left_button_up() -> Self {
        Self(Self::BUTTON1)
    }

    /// Create flags for right button down
    pub fn right_button_down() -> Self {
        Self(Self::DOWN | Self::BUTTON2)
    }

    /// Create flags for right button up
    pub fn right_button_up() -> Self {
        Self(Self::BUTTON2)
    }

    /// Create flags for middle button down
    pub fn middle_button_down() -> Self {
        Self(Self::DOWN | Self::BUTTON3)
    }

    /// Create flags for middle button up
    pub fn middle_button_up() -> Self {
        Self(Self::BUTTON3)
    }

    /// Create flags for vertical wheel (positive = up, negative = down)
    pub fn vertical_wheel(rotation: i16) -> Self {
        let mut flags = Self::WHEEL;
        if rotation < 0 {
            flags |= Self::WHEEL_NEGATIVE;
        }
        Self(flags)
    }

    /// Create flags for horizontal wheel (positive = right, negative = left)
    pub fn horizontal_wheel(rotation: i16) -> Self {
        let mut flags = Self::HWHEEL;
        if rotation < 0 {
            flags |= Self::WHEEL_NEGATIVE;
        }
        Self(flags)
    }

    /// Check if mouse moved
    pub fn is_move(&self) -> bool {
        self.0 & Self::MOVE != 0
    }

    /// Check if button is down
    pub fn is_down(&self) -> bool {
        self.0 & Self::DOWN != 0
    }

    /// Check if left button event
    pub fn is_button1(&self) -> bool {
        self.0 & Self::BUTTON1 != 0
    }

    /// Check if right button event
    pub fn is_button2(&self) -> bool {
        self.0 & Self::BUTTON2 != 0
    }

    /// Check if middle button event
    pub fn is_button3(&self) -> bool {
        self.0 & Self::BUTTON3 != 0
    }

    /// Check if vertical wheel event
    pub fn is_wheel(&self) -> bool {
        self.0 & Self::WHEEL != 0
    }

    /// Check if horizontal wheel event
    pub fn is_hwheel(&self) -> bool {
        self.0 & Self::HWHEEL != 0
    }

    /// Get raw value
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Mouse Event (MS-RDPBCGR 2.2.8.1.1.3.1.1.3)
///
/// Used to transmit mouse input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MouseEvent {
    /// Mouse event flags
    pub flags: MouseFlags,
    /// X position (0-based)
    pub x_pos: u16,
    /// Y position (0-based)
    pub y_pos: u16,
}

impl MouseEvent {
    /// Create new mouse event
    pub fn new(flags: MouseFlags, x_pos: u16, y_pos: u16) -> Self {
        Self { flags, x_pos, y_pos }
    }

    /// Create mouse move event
    pub fn move_to(x_pos: u16, y_pos: u16) -> Self {
        Self::new(MouseFlags::move_event(), x_pos, y_pos)
    }

    /// Create left button down event
    pub fn left_down(x_pos: u16, y_pos: u16) -> Self {
        Self::new(MouseFlags::left_button_down(), x_pos, y_pos)
    }

    /// Create left button up event
    pub fn left_up(x_pos: u16, y_pos: u16) -> Self {
        Self::new(MouseFlags::left_button_up(), x_pos, y_pos)
    }

    /// Create right button down event
    pub fn right_down(x_pos: u16, y_pos: u16) -> Self {
        Self::new(MouseFlags::right_button_down(), x_pos, y_pos)
    }

    /// Create right button up event
    pub fn right_up(x_pos: u16, y_pos: u16) -> Self {
        Self::new(MouseFlags::right_button_up(), x_pos, y_pos)
    }

    /// Event data size (6 bytes)
    pub const SIZE: usize = 6;

    /// Encode mouse event
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.flags.as_u16())?;
        buffer.write_u16::<LittleEndian>(self.x_pos)?;
        buffer.write_u16::<LittleEndian>(self.y_pos)?;
        Ok(())
    }

    /// Decode mouse event
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let flags = MouseFlags::new(buffer.read_u16::<LittleEndian>()?);
        let x_pos = buffer.read_u16::<LittleEndian>()?;
        let y_pos = buffer.read_u16::<LittleEndian>()?;

        Ok(Self { flags, x_pos, y_pos })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

/// Extended Mouse Event Flags (MS-RDPBCGR 2.2.8.1.1.3.1.1.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtendedMouseFlags(u16);

impl ExtendedMouseFlags {
    /// Button 1 (left button)
    pub const BUTTON1: u16 = 0x0001;
    /// Button 2 (right button)
    pub const BUTTON2: u16 = 0x0002;
    /// Button pressed
    pub const DOWN: u16 = 0x0010;
    /// Mouse moved
    pub const MOVE: u16 = 0x0020;
    /// X button 1 (back button)
    pub const XBUTTON1: u16 = 0x0004;
    /// X button 2 (forward button)
    pub const XBUTTON2: u16 = 0x0008;

    /// Create new extended mouse flags
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    /// Create flags for mouse move
    pub fn move_event() -> Self {
        Self(Self::MOVE)
    }

    /// Create flags for button 1 down
    pub fn button1_down() -> Self {
        Self(Self::DOWN | Self::BUTTON1)
    }

    /// Create flags for button 1 up
    pub fn button1_up() -> Self {
        Self(Self::BUTTON1)
    }

    /// Create flags for button 2 down
    pub fn button2_down() -> Self {
        Self(Self::DOWN | Self::BUTTON2)
    }

    /// Create flags for button 2 up
    pub fn button2_up() -> Self {
        Self(Self::BUTTON2)
    }

    /// Check if mouse moved
    pub fn is_move(&self) -> bool {
        self.0 & Self::MOVE != 0
    }

    /// Check if button is down
    pub fn is_down(&self) -> bool {
        self.0 & Self::DOWN != 0
    }

    /// Get raw value
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Extended Mouse Event (MS-RDPBCGR 2.2.8.1.1.3.1.1.4)
///
/// Extended mouse input (supports extra buttons)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedMouseEvent {
    /// Extended mouse event flags
    pub flags: ExtendedMouseFlags,
    /// X position (0-based)
    pub x_pos: u16,
    /// Y position (0-based)
    pub y_pos: u16,
}

impl ExtendedMouseEvent {
    /// Create new extended mouse event
    pub fn new(flags: ExtendedMouseFlags, x_pos: u16, y_pos: u16) -> Self {
        Self { flags, x_pos, y_pos }
    }

    /// Create mouse move event
    pub fn move_to(x_pos: u16, y_pos: u16) -> Self {
        Self::new(ExtendedMouseFlags::move_event(), x_pos, y_pos)
    }

    /// Event data size (6 bytes)
    pub const SIZE: usize = 6;

    /// Encode extended mouse event
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.flags.as_u16())?;
        buffer.write_u16::<LittleEndian>(self.x_pos)?;
        buffer.write_u16::<LittleEndian>(self.y_pos)?;
        Ok(())
    }

    /// Decode extended mouse event
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let flags = ExtendedMouseFlags::new(buffer.read_u16::<LittleEndian>()?);
        let x_pos = buffer.read_u16::<LittleEndian>()?;
        let y_pos = buffer.read_u16::<LittleEndian>()?;

        Ok(Self { flags, x_pos, y_pos })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

/// Sync Event (MS-RDPBCGR 2.2.8.1.1.3.1.1.5)
///
/// Synchronize keyboard state (LED indicators)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncEvent {
    /// Synchronization flags (toggle states)
    pub flags: u16,
    /// Padding (must be 0)
    pub pad: u16,
}

impl SyncEvent {
    /// Scroll Lock LED on
    pub const SCROLL_LOCK: u16 = 0x0001;
    /// Num Lock LED on
    pub const NUM_LOCK: u16 = 0x0002;
    /// Caps Lock LED on
    pub const CAPS_LOCK: u16 = 0x0004;
    /// Kana Lock LED on
    pub const KANA_LOCK: u16 = 0x0008;

    /// Create new sync event
    pub fn new(flags: u16) -> Self {
        Self { flags, pad: 0 }
    }

    /// Event data size (4 bytes)
    pub const SIZE: usize = 4;

    /// Encode sync event
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.flags)?;
        buffer.write_u16::<LittleEndian>(self.pad)?;
        Ok(())
    }

    /// Decode sync event
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let flags = buffer.read_u16::<LittleEndian>()?;
        let pad = buffer.read_u16::<LittleEndian>()?;

        Ok(Self { flags, pad })
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
    fn test_mouse_flags() {
        let flags = MouseFlags::move_event();
        assert!(flags.is_move());
        assert!(!flags.is_down());

        let flags = MouseFlags::left_button_down();
        assert!(flags.is_button1());
        assert!(flags.is_down());

        let flags = MouseFlags::right_button_up();
        assert!(flags.is_button2());
        assert!(!flags.is_down());
    }

    #[test]
    fn test_mouse_event_encode_decode() {
        let event = MouseEvent::move_to(100, 200);

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), MouseEvent::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = MouseEvent::decode(&mut cursor).unwrap();

        assert_eq!(decoded, event);
        assert_eq!(decoded.x_pos, 100);
        assert_eq!(decoded.y_pos, 200);
        assert!(decoded.flags.is_move());
    }

    #[test]
    fn test_mouse_event_button_down() {
        let event = MouseEvent::left_down(50, 75);

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = MouseEvent::decode(&mut cursor).unwrap();

        assert_eq!(decoded.x_pos, 50);
        assert_eq!(decoded.y_pos, 75);
        assert!(decoded.flags.is_button1());
        assert!(decoded.flags.is_down());
    }

    #[test]
    fn test_mouse_event_button_up() {
        let event = MouseEvent::right_up(120, 240);

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = MouseEvent::decode(&mut cursor).unwrap();

        assert!(decoded.flags.is_button2());
        assert!(!decoded.flags.is_down());
    }

    #[test]
    fn test_extended_mouse_flags() {
        let flags = ExtendedMouseFlags::move_event();
        assert!(flags.is_move());

        let flags = ExtendedMouseFlags::button1_down();
        assert!(flags.is_down());
    }

    #[test]
    fn test_extended_mouse_event_encode_decode() {
        let event = ExtendedMouseEvent::move_to(300, 400);

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), ExtendedMouseEvent::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = ExtendedMouseEvent::decode(&mut cursor).unwrap();

        assert_eq!(decoded, event);
        assert_eq!(decoded.x_pos, 300);
        assert_eq!(decoded.y_pos, 400);
        assert!(decoded.flags.is_move());
    }

    #[test]
    fn test_sync_event_encode_decode() {
        let event = SyncEvent::new(SyncEvent::CAPS_LOCK | SyncEvent::NUM_LOCK);

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), SyncEvent::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = SyncEvent::decode(&mut cursor).unwrap();

        assert_eq!(decoded, event);
        assert_eq!(decoded.flags, SyncEvent::CAPS_LOCK | SyncEvent::NUM_LOCK);
    }

    #[test]
    fn test_mouse_wheel() {
        let flags = MouseFlags::vertical_wheel(120);
        assert!(flags.is_wheel());
        assert!(!flags.is_hwheel());

        let flags = MouseFlags::vertical_wheel(-120);
        assert!(flags.is_wheel());
        assert_eq!(flags.0 & MouseFlags::WHEEL_NEGATIVE, MouseFlags::WHEEL_NEGATIVE);

        let flags = MouseFlags::horizontal_wheel(120);
        assert!(flags.is_hwheel());
    }
}
