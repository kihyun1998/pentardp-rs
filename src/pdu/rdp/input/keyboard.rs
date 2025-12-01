use crate::pdu::Result;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Keyboard Event Flags (MS-RDPBCGR 2.2.8.1.1.3.1.1.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardFlags(u16);

impl KeyboardFlags {
    /// Key released
    pub const RELEASE: u16 = 0x8000;
    /// Extended scancode (e.g., arrow keys, Insert, Delete)
    pub const EXTENDED: u16 = 0x0100;
    /// Extended1 scancode (e.g., Pause key)
    pub const EXTENDED1: u16 = 0x0200;

    /// Create new keyboard flags
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    /// Create flags for key down event
    pub fn key_down() -> Self {
        Self(0)
    }

    /// Create flags for key up event
    pub fn key_up() -> Self {
        Self(Self::RELEASE)
    }

    /// Create flags for extended key down
    pub fn extended_key_down() -> Self {
        Self(Self::EXTENDED)
    }

    /// Create flags for extended key up
    pub fn extended_key_up() -> Self {
        Self(Self::EXTENDED | Self::RELEASE)
    }

    /// Check if key is released
    pub fn is_release(&self) -> bool {
        self.0 & Self::RELEASE != 0
    }

    /// Check if key is extended
    pub fn is_extended(&self) -> bool {
        self.0 & Self::EXTENDED != 0
    }

    /// Get raw value
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Keyboard Event (SlowPath) (MS-RDPBCGR 2.2.8.1.1.3.1.1.1)
///
/// Used to transmit keyboard input (scancode-based)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardEvent {
    /// Keyboard event flags
    pub flags: KeyboardFlags,
    /// Scancode (e.g., 0x1E for 'A')
    pub key_code: u16,
    /// Padding (must be 0)
    pub pad: u16,
}

impl KeyboardEvent {
    /// Create new keyboard event
    pub fn new(flags: KeyboardFlags, key_code: u16) -> Self {
        Self {
            flags,
            key_code,
            pad: 0,
        }
    }

    /// Create key down event
    pub fn key_down(key_code: u16) -> Self {
        Self::new(KeyboardFlags::key_down(), key_code)
    }

    /// Create key up event
    pub fn key_up(key_code: u16) -> Self {
        Self::new(KeyboardFlags::key_up(), key_code)
    }

    /// Create extended key down event
    pub fn extended_key_down(key_code: u16) -> Self {
        Self::new(KeyboardFlags::extended_key_down(), key_code)
    }

    /// Create extended key up event
    pub fn extended_key_up(key_code: u16) -> Self {
        Self::new(KeyboardFlags::extended_key_up(), key_code)
    }

    /// Event data size (6 bytes)
    pub const SIZE: usize = 6;

    /// Encode keyboard event
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.flags.as_u16())?;
        buffer.write_u16::<LittleEndian>(self.key_code)?;
        buffer.write_u16::<LittleEndian>(self.pad)?;
        Ok(())
    }

    /// Decode keyboard event
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let flags = KeyboardFlags::new(buffer.read_u16::<LittleEndian>()?);
        let key_code = buffer.read_u16::<LittleEndian>()?;
        let pad = buffer.read_u16::<LittleEndian>()?;

        Ok(Self {
            flags,
            key_code,
            pad,
        })
    }

    /// Return size
    pub fn size(&self) -> usize {
        Self::SIZE
    }
}

/// Unicode Keyboard Event Flags (MS-RDPBCGR 2.2.8.1.1.3.1.1.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnicodeKeyboardFlags(u16);

impl UnicodeKeyboardFlags {
    /// Key released
    pub const RELEASE: u16 = 0x8000;

    /// Create new unicode keyboard flags
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    /// Create flags for key down
    pub fn key_down() -> Self {
        Self(0)
    }

    /// Create flags for key up
    pub fn key_up() -> Self {
        Self(Self::RELEASE)
    }

    /// Check if key is released
    pub fn is_release(&self) -> bool {
        self.0 & Self::RELEASE != 0
    }

    /// Get raw value
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

/// Unicode Keyboard Event (MS-RDPBCGR 2.2.8.1.1.3.1.1.2)
///
/// Used to transmit Unicode keyboard input
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnicodeKeyboardEvent {
    /// Unicode keyboard event flags
    pub flags: UnicodeKeyboardFlags,
    /// Unicode character code (UTF-16)
    pub unicode_code: u16,
    /// Padding (must be 0)
    pub pad: u16,
}

impl UnicodeKeyboardEvent {
    /// Create new unicode keyboard event
    pub fn new(flags: UnicodeKeyboardFlags, unicode_code: u16) -> Self {
        Self {
            flags,
            unicode_code,
            pad: 0,
        }
    }

    /// Create unicode key down event
    pub fn key_down(unicode_code: u16) -> Self {
        Self::new(UnicodeKeyboardFlags::key_down(), unicode_code)
    }

    /// Create unicode key up event
    pub fn key_up(unicode_code: u16) -> Self {
        Self::new(UnicodeKeyboardFlags::key_up(), unicode_code)
    }

    /// Event data size (6 bytes)
    pub const SIZE: usize = 6;

    /// Encode unicode keyboard event
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.flags.as_u16())?;
        buffer.write_u16::<LittleEndian>(self.unicode_code)?;
        buffer.write_u16::<LittleEndian>(self.pad)?;
        Ok(())
    }

    /// Decode unicode keyboard event
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let flags = UnicodeKeyboardFlags::new(buffer.read_u16::<LittleEndian>()?);
        let unicode_code = buffer.read_u16::<LittleEndian>()?;
        let pad = buffer.read_u16::<LittleEndian>()?;

        Ok(Self {
            flags,
            unicode_code,
            pad,
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
    fn test_keyboard_flags() {
        let flags = KeyboardFlags::key_down();
        assert_eq!(flags.as_u16(), 0);
        assert!(!flags.is_release());

        let flags = KeyboardFlags::key_up();
        assert!(flags.is_release());
        assert_eq!(flags.as_u16() & KeyboardFlags::RELEASE, KeyboardFlags::RELEASE);

        let flags = KeyboardFlags::extended_key_down();
        assert!(flags.is_extended());
        assert!(!flags.is_release());

        let flags = KeyboardFlags::extended_key_up();
        assert!(flags.is_extended());
        assert!(flags.is_release());
    }

    #[test]
    fn test_keyboard_event_encode_decode() {
        let event = KeyboardEvent::key_down(0x1E); // 'A' scancode

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), KeyboardEvent::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = KeyboardEvent::decode(&mut cursor).unwrap();

        assert_eq!(decoded, event);
        assert_eq!(decoded.key_code, 0x1E);
        assert!(!decoded.flags.is_release());
    }

    #[test]
    fn test_keyboard_event_key_up() {
        let event = KeyboardEvent::key_up(0x2C); // 'Z' scancode

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = KeyboardEvent::decode(&mut cursor).unwrap();

        assert_eq!(decoded.key_code, 0x2C);
        assert!(decoded.flags.is_release());
    }

    #[test]
    fn test_keyboard_event_extended() {
        let event = KeyboardEvent::extended_key_down(0x48); // Up arrow

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = KeyboardEvent::decode(&mut cursor).unwrap();

        assert_eq!(decoded.key_code, 0x48);
        assert!(decoded.flags.is_extended());
        assert!(!decoded.flags.is_release());
    }

    #[test]
    fn test_unicode_keyboard_flags() {
        let flags = UnicodeKeyboardFlags::key_down();
        assert_eq!(flags.as_u16(), 0);
        assert!(!flags.is_release());

        let flags = UnicodeKeyboardFlags::key_up();
        assert!(flags.is_release());
    }

    #[test]
    fn test_unicode_keyboard_event_encode_decode() {
        let event = UnicodeKeyboardEvent::key_down(0x0041); // 'A' Unicode

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), UnicodeKeyboardEvent::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = UnicodeKeyboardEvent::decode(&mut cursor).unwrap();

        assert_eq!(decoded, event);
        assert_eq!(decoded.unicode_code, 0x0041);
        assert!(!decoded.flags.is_release());
    }

    #[test]
    fn test_unicode_keyboard_event_key_up() {
        let event = UnicodeKeyboardEvent::key_up(0x4E2D); // Chinese character

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = UnicodeKeyboardEvent::decode(&mut cursor).unwrap();

        assert_eq!(decoded.unicode_code, 0x4E2D);
        assert!(decoded.flags.is_release());
    }

    #[test]
    fn test_keyboard_event_roundtrip() {
        let event = KeyboardEvent::new(KeyboardFlags::new(0x8100), 0x1C); // Extended + Release, Enter

        let mut buffer = Vec::new();
        event.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = KeyboardEvent::decode(&mut cursor).unwrap();

        assert_eq!(event, decoded);
    }
}
