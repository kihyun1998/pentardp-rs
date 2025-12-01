// RDP Input PDUs
pub mod keyboard;
pub mod mouse;

pub use keyboard::{KeyboardEvent, KeyboardFlags, UnicodeKeyboardEvent, UnicodeKeyboardFlags};
pub use mouse::{ExtendedMouseEvent, ExtendedMouseFlags, MouseEvent, MouseFlags, SyncEvent};

use crate::pdu::{Pdu, PduError, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

/// Input Event Type (MS-RDPBCGR 2.2.8.1.1.3.1.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum InputEventType {
    /// Synchronize event
    Sync = 0x0000,
    /// Unused (reserved)
    Unused = 0x0002,
    /// Keyboard event (scancode)
    Scancode = 0x0004,
    /// Unicode keyboard event
    Unicode = 0x0005,
    /// Mouse event
    Mouse = 0x8001,
    /// Extended mouse event
    ExtendedMouse = 0x8002,
}

impl InputEventType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0000 => Some(InputEventType::Sync),
            0x0002 => Some(InputEventType::Unused),
            0x0004 => Some(InputEventType::Scancode),
            0x0005 => Some(InputEventType::Unicode),
            0x8001 => Some(InputEventType::Mouse),
            0x8002 => Some(InputEventType::ExtendedMouse),
            _ => None,
        }
    }

    pub fn as_u16(self) -> u16 {
        self as u16
    }
}

/// Input Event (MS-RDPBCGR 2.2.8.1.1.3.1.1)
///
/// Variable-length structure containing input event data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEvent {
    /// Keyboard event (SlowPath scancode)
    Keyboard(KeyboardEvent),
    /// Unicode keyboard event
    Unicode(UnicodeKeyboardEvent),
    /// Mouse event
    Mouse(MouseEvent),
    /// Extended mouse event
    ExtendedMouse(ExtendedMouseEvent),
    /// Synchronize event
    Sync(SyncEvent),
}

impl InputEvent {
    /// Event data size (6 bytes: 2 for eventTime + 2 for messageType + 2 for pad/data)
    pub const BASE_SIZE: usize = 4;

    /// Get event type
    pub fn event_type(&self) -> InputEventType {
        match self {
            InputEvent::Keyboard(_) => InputEventType::Scancode,
            InputEvent::Unicode(_) => InputEventType::Unicode,
            InputEvent::Mouse(_) => InputEventType::Mouse,
            InputEvent::ExtendedMouse(_) => InputEventType::ExtendedMouse,
            InputEvent::Sync(_) => InputEventType::Sync,
        }
    }

    /// Encode input event
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        // eventTime (2 bytes) - usually 0
        buffer.write_u16::<LittleEndian>(0)?;
        // messageType (2 bytes)
        buffer.write_u16::<LittleEndian>(self.event_type().as_u16())?;

        // Event-specific data
        match self {
            InputEvent::Keyboard(event) => event.encode(buffer)?,
            InputEvent::Unicode(event) => event.encode(buffer)?,
            InputEvent::Mouse(event) => event.encode(buffer)?,
            InputEvent::ExtendedMouse(event) => event.encode(buffer)?,
            InputEvent::Sync(event) => event.encode(buffer)?,
        }

        Ok(())
    }

    /// Decode input event
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let _event_time = buffer.read_u16::<LittleEndian>()?;
        let message_type = buffer.read_u16::<LittleEndian>()?;

        let event_type = InputEventType::from_u16(message_type).ok_or_else(|| {
            PduError::ParseError(format!("Invalid input event type: {:#x}", message_type))
        })?;

        match event_type {
            InputEventType::Scancode => Ok(InputEvent::Keyboard(KeyboardEvent::decode(buffer)?)),
            InputEventType::Unicode => Ok(InputEvent::Unicode(UnicodeKeyboardEvent::decode(buffer)?)),
            InputEventType::Mouse => Ok(InputEvent::Mouse(MouseEvent::decode(buffer)?)),
            InputEventType::ExtendedMouse => {
                Ok(InputEvent::ExtendedMouse(ExtendedMouseEvent::decode(buffer)?))
            }
            InputEventType::Sync => Ok(InputEvent::Sync(SyncEvent::decode(buffer)?)),
            InputEventType::Unused => {
                // Skip unused event data (6 bytes)
                let mut _unused = [0u8; 6];
                buffer.read_exact(&mut _unused)?;
                Err(PduError::ParseError("Unused event type encountered".to_string()))
            }
        }
    }

    /// Return size (eventTime + messageType + event data)
    pub fn size(&self) -> usize {
        Self::BASE_SIZE
            + match self {
                InputEvent::Keyboard(event) => event.size(),
                InputEvent::Unicode(event) => event.size(),
                InputEvent::Mouse(event) => event.size(),
                InputEvent::ExtendedMouse(event) => event.size(),
                InputEvent::Sync(event) => event.size(),
            }
    }
}

/// Input Event PDU (MS-RDPBCGR 2.2.8.1.1.3)
///
/// Sent from client to server to communicate input events
///
/// This PDU is sent within a Share Data Header (pdu_type2 = Input)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEventPdu {
    /// Number of input events
    pub num_events: u16,
    /// Input events array
    pub events: Vec<InputEvent>,
}

impl InputEventPdu {
    /// Create new Input Event PDU
    pub fn new(events: Vec<InputEvent>) -> Self {
        let num_events = events.len() as u16;
        Self { num_events, events }
    }

    /// Create PDU with single event
    pub fn single(event: InputEvent) -> Self {
        Self::new(vec![event])
    }

    /// Minimum size (2 bytes for num_events + 2 bytes padding)
    pub const MIN_SIZE: usize = 4;
}

impl Pdu for InputEventPdu {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        // numEvents (2 bytes)
        buffer.write_u16::<LittleEndian>(self.num_events)?;
        // pad2Octets (2 bytes)
        buffer.write_u16::<LittleEndian>(0)?;

        // Encode all events
        for event in &self.events {
            event.encode(buffer)?;
        }

        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let num_events = buffer.read_u16::<LittleEndian>()?;
        let _pad = buffer.read_u16::<LittleEndian>()?;

        let mut events = Vec::with_capacity(num_events as usize);
        for _ in 0..num_events {
            events.push(InputEvent::decode(buffer)?);
        }

        Ok(Self { num_events, events })
    }

    fn size(&self) -> usize {
        Self::MIN_SIZE + self.events.iter().map(|e| e.size()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_input_event_type() {
        assert_eq!(InputEventType::Scancode.as_u16(), 0x0004);
        assert_eq!(
            InputEventType::from_u16(0x0004),
            Some(InputEventType::Scancode)
        );
        assert_eq!(InputEventType::from_u16(0xFFFF), None);
    }

    #[test]
    fn test_input_event_pdu_empty() {
        let pdu = InputEventPdu::new(vec![]);

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), InputEventPdu::MIN_SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = InputEventPdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.num_events, 0);
        assert_eq!(decoded.events.len(), 0);
    }
}
