use crate::pdu::Result;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

use super::sets::{CapabilitySetHeader, CapabilitySetType};

bitflags! {
    /// Input Flags (MS-RDPBCGR 2.2.7.1.6)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InputFlags: u16 {
        /// INPUT_FLAG_SCANCODES - Keyboard scancodes
        const SCANCODES = 0x0001;
        /// INPUT_FLAG_MOUSEX - Mouse extended buttons
        const MOUSEX = 0x0004;
        /// INPUT_FLAG_FASTPATH_INPUT - Fast-path input
        const FASTPATH_INPUT = 0x0008;
        /// INPUT_FLAG_UNICODE - Unicode keyboard events
        const UNICODE = 0x0010;
        /// INPUT_FLAG_FASTPATH_INPUT2 - Fast-path input 2
        const FASTPATH_INPUT2 = 0x0020;
        /// INPUT_FLAG_UNUSED1 - Unused
        const UNUSED1 = 0x0040;
        /// INPUT_FLAG_UNUSED2 - Unused
        const UNUSED2 = 0x0080;
        /// INPUT_FLAG_MOUSE_HWHEEL - Mouse horizontal wheel
        const MOUSE_HWHEEL = 0x0100;
    }
}

/// Input Capability Set (MS-RDPBCGR 2.2.7.1.6)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputCapability {
    /// Input flags
    pub input_flags: InputFlags,
    /// Keyboard layout (0 = US English)
    pub keyboard_layout: u32,
    /// Keyboard type (4 = IBM enhanced 101/102-key)
    pub keyboard_type: u32,
    /// Keyboard subtype
    pub keyboard_subtype: u32,
    /// Keyboard function key count (12 = standard)
    pub keyboard_function_key: u32,
    /// IME file name (64 bytes, Unicode)
    pub ime_file_name: String,
}

impl InputCapability {
    /// Create new Input Capability
    pub fn new() -> Self {
        Self {
            input_flags: InputFlags::SCANCODES
                | InputFlags::MOUSEX
                | InputFlags::FASTPATH_INPUT
                | InputFlags::UNICODE
                | InputFlags::FASTPATH_INPUT2,
            keyboard_layout: 0x0409, // US English
            keyboard_type: 4,         // IBM enhanced
            keyboard_subtype: 0,
            keyboard_function_key: 12,
            ime_file_name: String::new(),
        }
    }

    /// Data size (excluding header)
    pub const DATA_SIZE: usize = 84;
    /// IME file name size (64 bytes)
    pub const IME_FILE_NAME_SIZE: usize = 64;

    /// Encode capability set (with header)
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        let header = CapabilitySetHeader::new(
            CapabilitySetType::Input,
            (CapabilitySetHeader::SIZE + Self::DATA_SIZE) as u16,
        );
        header.encode(buffer)?;

        buffer.write_u16::<LittleEndian>(self.input_flags.bits())?;
        buffer.write_u16::<LittleEndian>(0)?; // padding
        buffer.write_u32::<LittleEndian>(self.keyboard_layout)?;
        buffer.write_u32::<LittleEndian>(self.keyboard_type)?;
        buffer.write_u32::<LittleEndian>(self.keyboard_subtype)?;
        buffer.write_u32::<LittleEndian>(self.keyboard_function_key)?;

        // IME file name (64 bytes, UTF-16LE)
        let mut ime_buf = [0u8; Self::IME_FILE_NAME_SIZE];
        encode_unicode_string(&self.ime_file_name, &mut ime_buf)?;
        buffer.write_all(&ime_buf)?;

        Ok(())
    }

    /// Decode capability data (without header)
    pub fn decode_data(buffer: &mut dyn Read, _data_len: usize) -> Result<Self> {
        let input_flags_bits = buffer.read_u16::<LittleEndian>()?;
        let input_flags = InputFlags::from_bits_truncate(input_flags_bits);

        let _padding = buffer.read_u16::<LittleEndian>()?;
        let keyboard_layout = buffer.read_u32::<LittleEndian>()?;
        let keyboard_type = buffer.read_u32::<LittleEndian>()?;
        let keyboard_subtype = buffer.read_u32::<LittleEndian>()?;
        let keyboard_function_key = buffer.read_u32::<LittleEndian>()?;

        let mut ime_buf = [0u8; Self::IME_FILE_NAME_SIZE];
        buffer.read_exact(&mut ime_buf)?;
        let ime_file_name = decode_unicode_string(&ime_buf);

        Ok(Self {
            input_flags,
            keyboard_layout,
            keyboard_type,
            keyboard_subtype,
            keyboard_function_key,
            ime_file_name,
        })
    }

    /// Get size (including header)
    pub fn size(&self) -> usize {
        CapabilitySetHeader::SIZE + Self::DATA_SIZE
    }
}

impl Default for InputCapability {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for Unicode encoding/decoding

fn encode_unicode_string(s: &str, buffer: &mut [u8]) -> Result<()> {
    let chars: Vec<u16> = s.encode_utf16().collect();
    let max_chars = buffer.len() / 2;

    for (i, &ch) in chars.iter().take(max_chars).enumerate() {
        let offset = i * 2;
        buffer[offset] = (ch & 0xFF) as u8;
        buffer[offset + 1] = (ch >> 8) as u8;
    }

    Ok(())
}

fn decode_unicode_string(buffer: &[u8]) -> String {
    let mut chars = Vec::new();

    for i in (0..buffer.len()).step_by(2) {
        if i + 1 >= buffer.len() {
            break;
        }

        let ch = u16::from_le_bytes([buffer[i], buffer[i + 1]]);
        if ch == 0 {
            break;
        }
        chars.push(ch);
    }

    String::from_utf16(&chars).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_input_flags() {
        let flags = InputFlags::SCANCODES | InputFlags::UNICODE;
        assert!(flags.contains(InputFlags::SCANCODES));
        assert!(flags.contains(InputFlags::UNICODE));
        assert!(!flags.contains(InputFlags::MOUSEX));
    }

    #[test]
    fn test_input_capability() {
        let cap = InputCapability::new();

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), cap.size());

        let mut cursor = Cursor::new(&buffer[CapabilitySetHeader::SIZE..]);
        let decoded = InputCapability::decode_data(&mut cursor, InputCapability::DATA_SIZE).unwrap();

        assert_eq!(decoded.keyboard_layout, 0x0409);
        assert_eq!(decoded.keyboard_type, 4);
        assert_eq!(decoded.keyboard_function_key, 12);
    }

    #[test]
    fn test_input_capability_roundtrip() {
        let cap = InputCapability::new();

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(&buffer[CapabilitySetHeader::SIZE..]);
        let decoded = InputCapability::decode_data(&mut cursor, InputCapability::DATA_SIZE).unwrap();

        assert_eq!(cap, decoded);
    }

    #[test]
    fn test_input_capability_with_ime() {
        let mut cap = InputCapability::new();
        cap.ime_file_name = "msime.ime".to_string();

        let mut buffer = Vec::new();
        cap.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(&buffer[CapabilitySetHeader::SIZE..]);
        let decoded = InputCapability::decode_data(&mut cursor, InputCapability::DATA_SIZE).unwrap();

        assert_eq!(decoded.ime_file_name, "msime.ime");
    }
}
