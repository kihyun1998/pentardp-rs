use crate::pdu::{Pdu, PduError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Control Action (MS-RDPBCGR 2.2.1.15.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum ControlAction {
    /// Request control
    RequestControl = 0x0001,
    /// Granted control
    GrantedControl = 0x0002,
    /// Detach
    Detach = 0x0003,
    /// Cooperate
    Cooperate = 0x0004,
}

impl ControlAction {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0001 => Some(ControlAction::RequestControl),
            0x0002 => Some(ControlAction::GrantedControl),
            0x0003 => Some(ControlAction::Detach),
            0x0004 => Some(ControlAction::Cooperate),
            _ => None,
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Synchronize PDU (MS-RDPBCGR 2.2.1.14)
///
/// Used to synchronize the client and server
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchronizePdu {
    /// Message type (SYNCMSGTYPE_SYNC = 1)
    pub message_type: u16,
    /// Target user (MCS channel ID, usually 1002 + user_id)
    pub target_user: u16,
}

impl SynchronizePdu {
    /// Message type constant (SYNCMSGTYPE_SYNC)
    pub const SYNCMSGTYPE_SYNC: u16 = 1;

    /// Create new Synchronize PDU
    pub fn new(target_user: u16) -> Self {
        Self {
            message_type: Self::SYNCMSGTYPE_SYNC,
            target_user,
        }
    }

    /// PDU data size (4 bytes)
    pub const SIZE: usize = 4;
}

impl Pdu for SynchronizePdu {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.message_type)?;
        buffer.write_u16::<LittleEndian>(self.target_user)?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let message_type = buffer.read_u16::<LittleEndian>()?;
        let target_user = buffer.read_u16::<LittleEndian>()?;

        Ok(Self {
            message_type,
            target_user,
        })
    }

    fn size(&self) -> usize {
        Self::SIZE
    }
}

/// Control PDU (MS-RDPBCGR 2.2.1.15)
///
/// Used for control and cooperation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPdu {
    /// Control action
    pub action: ControlAction,
    /// Grant ID (0 for all actions except GrantedControl)
    pub grant_id: u16,
    /// Control ID (0 for most actions)
    pub control_id: u32,
}

impl ControlPdu {
    /// Create Cooperate PDU
    pub fn cooperate() -> Self {
        Self {
            action: ControlAction::Cooperate,
            grant_id: 0,
            control_id: 0,
        }
    }

    /// Create Request Control PDU
    pub fn request_control() -> Self {
        Self {
            action: ControlAction::RequestControl,
            grant_id: 0,
            control_id: 0,
        }
    }

    /// Create Granted Control PDU
    pub fn granted_control(grant_id: u16) -> Self {
        Self {
            action: ControlAction::GrantedControl,
            grant_id,
            control_id: 1000,
        }
    }

    /// PDU data size (8 bytes)
    pub const SIZE: usize = 8;
}

impl Pdu for ControlPdu {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.action.as_u16())?;
        buffer.write_u16::<LittleEndian>(self.grant_id)?;
        buffer.write_u32::<LittleEndian>(self.control_id)?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let action_value = buffer.read_u16::<LittleEndian>()?;
        let action = ControlAction::from_u16(action_value).ok_or_else(|| {
            PduError::ParseError(format!("Invalid control action: {:#x}", action_value))
        })?;

        let grant_id = buffer.read_u16::<LittleEndian>()?;
        let control_id = buffer.read_u32::<LittleEndian>()?;

        Ok(Self {
            action,
            grant_id,
            control_id,
        })
    }

    fn size(&self) -> usize {
        Self::SIZE
    }
}

/// Font List PDU (MS-RDPBCGR 2.2.1.18)
///
/// Sent to inform the server about font list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontListPdu {
    /// Number of fonts (0)
    pub number_fonts: u16,
    /// Total number of fonts (0)
    pub total_num_fonts: u16,
    /// List flags (FONTLIST_FIRST | FONTLIST_LAST = 0x0003)
    pub list_flags: u16,
    /// Entry size (0x0032 = 50)
    pub entry_size: u16,
}

impl FontListPdu {
    /// List flags constant (FONTLIST_FIRST | FONTLIST_LAST)
    pub const FONTLIST_FIRST_AND_LAST: u16 = 0x0003;

    /// Entry size constant
    pub const ENTRY_SIZE: u16 = 0x0032;

    /// Create new Font List PDU
    pub fn new() -> Self {
        Self {
            number_fonts: 0,
            total_num_fonts: 0,
            list_flags: Self::FONTLIST_FIRST_AND_LAST,
            entry_size: Self::ENTRY_SIZE,
        }
    }

    /// PDU data size (8 bytes)
    pub const SIZE: usize = 8;
}

impl Default for FontListPdu {
    fn default() -> Self {
        Self::new()
    }
}

impl Pdu for FontListPdu {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.number_fonts)?;
        buffer.write_u16::<LittleEndian>(self.total_num_fonts)?;
        buffer.write_u16::<LittleEndian>(self.list_flags)?;
        buffer.write_u16::<LittleEndian>(self.entry_size)?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let number_fonts = buffer.read_u16::<LittleEndian>()?;
        let total_num_fonts = buffer.read_u16::<LittleEndian>()?;
        let list_flags = buffer.read_u16::<LittleEndian>()?;
        let entry_size = buffer.read_u16::<LittleEndian>()?;

        Ok(Self {
            number_fonts,
            total_num_fonts,
            list_flags,
            entry_size,
        })
    }

    fn size(&self) -> usize {
        Self::SIZE
    }
}

/// Font Map PDU (MS-RDPBCGR 2.2.1.22)
///
/// Sent by server in response to Font List PDU
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontMapPdu {
    /// Number of entries (0)
    pub number_entries: u16,
    /// Total number of entries (0)
    pub total_num_entries: u16,
    /// Map flags (FONTMAP_FIRST | FONTMAP_LAST = 0x0003)
    pub map_flags: u16,
    /// Entry size (0x0004)
    pub entry_size: u16,
}

impl FontMapPdu {
    /// Map flags constant (FONTMAP_FIRST | FONTMAP_LAST)
    pub const FONTMAP_FIRST_AND_LAST: u16 = 0x0003;

    /// Entry size constant
    pub const ENTRY_SIZE: u16 = 0x0004;

    /// Create new Font Map PDU
    pub fn new() -> Self {
        Self {
            number_entries: 0,
            total_num_entries: 0,
            map_flags: Self::FONTMAP_FIRST_AND_LAST,
            entry_size: Self::ENTRY_SIZE,
        }
    }

    /// PDU data size (8 bytes)
    pub const SIZE: usize = 8;
}

impl Default for FontMapPdu {
    fn default() -> Self {
        Self::new()
    }
}

impl Pdu for FontMapPdu {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u16::<LittleEndian>(self.number_entries)?;
        buffer.write_u16::<LittleEndian>(self.total_num_entries)?;
        buffer.write_u16::<LittleEndian>(self.map_flags)?;
        buffer.write_u16::<LittleEndian>(self.entry_size)?;
        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let number_entries = buffer.read_u16::<LittleEndian>()?;
        let total_num_entries = buffer.read_u16::<LittleEndian>()?;
        let map_flags = buffer.read_u16::<LittleEndian>()?;
        let entry_size = buffer.read_u16::<LittleEndian>()?;

        Ok(Self {
            number_entries,
            total_num_entries,
            map_flags,
            entry_size,
        })
    }

    fn size(&self) -> usize {
        Self::SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_control_action() {
        assert_eq!(ControlAction::Cooperate.as_u16(), 0x0004);
        assert_eq!(
            ControlAction::from_u16(0x0004),
            Some(ControlAction::Cooperate)
        );
        assert_eq!(ControlAction::from_u16(0xFFFF), None);
    }

    #[test]
    fn test_synchronize_pdu() {
        let pdu = SynchronizePdu::new(1003);

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), SynchronizePdu::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = SynchronizePdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.message_type, SynchronizePdu::SYNCMSGTYPE_SYNC);
        assert_eq!(decoded.target_user, 1003);
    }

    #[test]
    fn test_control_pdu_cooperate() {
        let pdu = ControlPdu::cooperate();

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), ControlPdu::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = ControlPdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.action, ControlAction::Cooperate);
        assert_eq!(decoded.grant_id, 0);
    }

    #[test]
    fn test_control_pdu_request_control() {
        let pdu = ControlPdu::request_control();

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ControlPdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.action, ControlAction::RequestControl);
    }

    #[test]
    fn test_control_pdu_granted_control() {
        let pdu = ControlPdu::granted_control(1234);

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ControlPdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.action, ControlAction::GrantedControl);
        assert_eq!(decoded.grant_id, 1234);
        assert_eq!(decoded.control_id, 1000);
    }

    #[test]
    fn test_font_list_pdu() {
        let pdu = FontListPdu::new();

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), FontListPdu::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = FontListPdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.number_fonts, 0);
        assert_eq!(decoded.list_flags, FontListPdu::FONTLIST_FIRST_AND_LAST);
        assert_eq!(decoded.entry_size, FontListPdu::ENTRY_SIZE);
    }

    #[test]
    fn test_font_map_pdu() {
        let pdu = FontMapPdu::new();

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), FontMapPdu::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = FontMapPdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.number_entries, 0);
        assert_eq!(decoded.map_flags, FontMapPdu::FONTMAP_FIRST_AND_LAST);
        assert_eq!(decoded.entry_size, FontMapPdu::ENTRY_SIZE);
    }

    #[test]
    fn test_synchronize_pdu_roundtrip() {
        let pdu = SynchronizePdu::new(1004);

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = SynchronizePdu::decode(&mut cursor).unwrap();

        assert_eq!(pdu, decoded);
    }
}
