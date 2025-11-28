use crate::pdu::{Pdu, PduError, Result};
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

bitflags! {
    /// Client Info PDU Flags (MS-RDPBCGR 2.2.1.11.1.1)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ClientInfoFlags: u32 {
        /// INFO_MOUSE - Mouse input supported
        const MOUSE = 0x0000_0001;
        /// INFO_DISABLECTRLALTDEL - Disable Ctrl+Alt+Del
        const DISABLECTRLALTDEL = 0x0000_0002;
        /// INFO_AUTOLOGON - Auto logon
        const AUTOLOGON = 0x0000_0008;
        /// INFO_UNICODE - Unicode data
        const UNICODE = 0x0000_0010;
        /// INFO_MAXIMIZESHELL - Maximize shell
        const MAXIMIZESHELL = 0x0000_0020;
        /// INFO_LOGONNOTIFY - Logon notify
        const LOGONNOTIFY = 0x0000_0040;
        /// INFO_COMPRESSION - Compression supported
        const COMPRESSION = 0x0000_0080;
        /// INFO_ENABLEWINDOWSKEY - Enable Windows key
        const ENABLEWINDOWSKEY = 0x0000_0100;
        /// INFO_REMOTECONSOLEAUDIO - Remote console audio
        const REMOTECONSOLEAUDIO = 0x0000_2000;
        /// INFO_FORCE_ENCRYPTED_CS_PDU - Force encrypted Client-to-Server PDUs
        const FORCE_ENCRYPTED_CS_PDU = 0x0000_4000;
        /// INFO_RAIL - Remote Applications Integrated Locally
        const RAIL = 0x0000_8000;
        /// INFO_LOGONERRORS - Logon errors
        const LOGONERRORS = 0x0001_0000;
        /// INFO_MOUSE_HAS_WHEEL - Mouse has wheel
        const MOUSE_HAS_WHEEL = 0x0002_0000;
        /// INFO_PASSWORD_IS_SC_PIN - Password is smartcard PIN
        const PASSWORD_IS_SC_PIN = 0x0004_0000;
        /// INFO_NOAUDIOPLAYBACK - No audio playback
        const NOAUDIOPLAYBACK = 0x0008_0000;
        /// INFO_USING_SAVED_CREDS - Using saved credentials
        const USING_SAVED_CREDS = 0x0010_0000;
        /// INFO_AUDIOCAPTURE - Audio capture
        const AUDIOCAPTURE = 0x0020_0000;
        /// INFO_VIDEO_DISABLE - Video disable
        const VIDEO_DISABLE = 0x0040_0000;
        /// INFO_HIDEF_RAIL_SUPPORTED - HiDef RemoteApp supported
        const HIDEF_RAIL_SUPPORTED = 0x0200_0000;
    }
}

bitflags! {
    /// Performance Flags (MS-RDPBCGR 2.2.1.11.1.1.1)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PerformanceFlags: u32 {
        /// PERF_DISABLE_WALLPAPER - Disable wallpaper
        const DISABLE_WALLPAPER = 0x0000_0001;
        /// PERF_DISABLE_FULLWINDOWDRAG - Disable full window drag
        const DISABLE_FULLWINDOWDRAG = 0x0000_0002;
        /// PERF_DISABLE_MENUANIMATIONS - Disable menu animations
        const DISABLE_MENUANIMATIONS = 0x0000_0004;
        /// PERF_DISABLE_THEMING - Disable theming
        const DISABLE_THEMING = 0x0000_0008;
        /// PERF_DISABLE_CURSOR_SHADOW - Disable cursor shadow
        const DISABLE_CURSOR_SHADOW = 0x0000_0020;
        /// PERF_DISABLE_CURSORSETTINGS - Disable cursor blink
        const DISABLE_CURSORSETTINGS = 0x0000_0040;
        /// PERF_ENABLE_FONT_SMOOTHING - Enable font smoothing
        const ENABLE_FONT_SMOOTHING = 0x0000_0080;
        /// PERF_ENABLE_DESKTOP_COMPOSITION - Enable desktop composition
        const ENABLE_DESKTOP_COMPOSITION = 0x0000_0100;
    }
}

/// Time Zone Information (MS-RDPBCGR 2.2.1.11.1.1.1)
///
/// 172 bytes total
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeZoneInformation {
    /// Bias in minutes (UTC = local time + bias)
    pub bias: u32,
    /// Standard name (64 bytes, UTF-16LE, null-terminated)
    pub standard_name: String,
    /// Standard date (16 bytes)
    pub standard_date: [u8; 16],
    /// Standard bias in minutes
    pub standard_bias: u32,
    /// Daylight name (64 bytes, UTF-16LE, null-terminated)
    pub daylight_name: String,
    /// Daylight date (16 bytes)
    pub daylight_date: [u8; 16],
    /// Daylight bias in minutes
    pub daylight_bias: u32,
}

impl TimeZoneInformation {
    /// Size in bytes (172)
    pub const SIZE: usize = 172;

    /// Create new TimeZoneInformation with UTC
    pub fn utc() -> Self {
        Self {
            bias: 0,
            standard_name: String::new(),
            standard_date: [0; 16],
            standard_bias: 0,
            daylight_name: String::new(),
            daylight_date: [0; 16],
            daylight_bias: 0,
        }
    }

    /// Encode to buffer
    pub fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        buffer.write_u32::<LittleEndian>(self.bias)?;

        // Standard name (64 bytes, UTF-16LE)
        let mut name_buf = [0u8; 64];
        encode_unicode_string(&self.standard_name, &mut name_buf, 32)?;
        buffer.write_all(&name_buf)?;

        buffer.write_all(&self.standard_date)?;
        buffer.write_u32::<LittleEndian>(self.standard_bias)?;

        // Daylight name (64 bytes, UTF-16LE)
        let mut name_buf = [0u8; 64];
        encode_unicode_string(&self.daylight_name, &mut name_buf, 32)?;
        buffer.write_all(&name_buf)?;

        buffer.write_all(&self.daylight_date)?;
        buffer.write_u32::<LittleEndian>(self.daylight_bias)?;

        Ok(())
    }

    /// Decode from buffer
    pub fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let bias = buffer.read_u32::<LittleEndian>()?;

        let mut standard_name_buf = [0u8; 64];
        buffer.read_exact(&mut standard_name_buf)?;
        let standard_name = decode_unicode_string(&standard_name_buf);

        let mut standard_date = [0u8; 16];
        buffer.read_exact(&mut standard_date)?;
        let standard_bias = buffer.read_u32::<LittleEndian>()?;

        let mut daylight_name_buf = [0u8; 64];
        buffer.read_exact(&mut daylight_name_buf)?;
        let daylight_name = decode_unicode_string(&daylight_name_buf);

        let mut daylight_date = [0u8; 16];
        buffer.read_exact(&mut daylight_date)?;
        let daylight_bias = buffer.read_u32::<LittleEndian>()?;

        Ok(Self {
            bias,
            standard_name,
            standard_date,
            standard_bias,
            daylight_name,
            daylight_date,
            daylight_bias,
        })
    }
}

impl Default for TimeZoneInformation {
    fn default() -> Self {
        Self::utc()
    }
}

/// Client Info PDU (MS-RDPBCGR 2.2.1.11)
///
/// 클라이언트 인증 정보 및 설정
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInfoPdu {
    /// Code page (보통 0)
    pub code_page: u32,
    /// Flags
    pub flags: ClientInfoFlags,
    /// Domain name
    pub domain: String,
    /// User name
    pub user_name: String,
    /// Password
    pub password: String,
    /// Alternate shell
    pub alternate_shell: String,
    /// Working directory
    pub working_dir: String,
    /// Extended info (optional)
    pub extended_info: Option<ExtendedInfo>,
}

/// Extended Client Info (MS-RDPBCGR 2.2.1.11.1.1.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedInfo {
    /// Client address family (AF_INET=2, AF_INET6=23)
    pub client_address_family: u16,
    /// Client address
    pub client_address: String,
    /// Client directory
    pub client_dir: String,
    /// Client time zone
    pub client_time_zone: TimeZoneInformation,
    /// Client session ID
    pub client_session_id: u32,
    /// Performance flags
    pub performance_flags: PerformanceFlags,
}

impl ClientInfoPdu {
    /// Create new Client Info PDU
    pub fn new(user_name: String, password: String) -> Self {
        Self {
            code_page: 0,
            flags: ClientInfoFlags::MOUSE
                | ClientInfoFlags::UNICODE
                | ClientInfoFlags::LOGONNOTIFY
                | ClientInfoFlags::MAXIMIZESHELL
                | ClientInfoFlags::ENABLEWINDOWSKEY
                | ClientInfoFlags::DISABLECTRLALTDEL,
            domain: String::new(),
            user_name,
            password,
            alternate_shell: String::new(),
            working_dir: String::new(),
            extended_info: None,
        }
    }

    /// Set extended info
    pub fn with_extended_info(mut self, extended_info: ExtendedInfo) -> Self {
        self.extended_info = Some(extended_info);
        self
    }

    /// Set domain
    pub fn with_domain(mut self, domain: String) -> Self {
        self.domain = domain;
        self
    }
}

impl Pdu for ClientInfoPdu {
    fn encode(&self, buffer: &mut dyn Write) -> Result<()> {
        // Code page and flags
        buffer.write_u32::<LittleEndian>(self.code_page)?;
        buffer.write_u32::<LittleEndian>(self.flags.bits())?;

        // String lengths (in bytes, including null terminator)
        let domain_bytes = encode_string_length(&self.domain);
        let user_bytes = encode_string_length(&self.user_name);
        let password_bytes = encode_string_length(&self.password);
        let alt_shell_bytes = encode_string_length(&self.alternate_shell);
        let work_dir_bytes = encode_string_length(&self.working_dir);

        buffer.write_u16::<LittleEndian>(domain_bytes)?;
        buffer.write_u16::<LittleEndian>(user_bytes)?;
        buffer.write_u16::<LittleEndian>(password_bytes)?;
        buffer.write_u16::<LittleEndian>(alt_shell_bytes)?;
        buffer.write_u16::<LittleEndian>(work_dir_bytes)?;

        // Write strings (UTF-16LE with null terminator)
        write_unicode_string(buffer, &self.domain)?;
        write_unicode_string(buffer, &self.user_name)?;
        write_unicode_string(buffer, &self.password)?;
        write_unicode_string(buffer, &self.alternate_shell)?;
        write_unicode_string(buffer, &self.working_dir)?;

        // Extended info (if present)
        if let Some(ref ext) = self.extended_info {
            buffer.write_u16::<LittleEndian>(ext.client_address_family)?;

            let client_addr_bytes = encode_string_length(&ext.client_address);
            buffer.write_u16::<LittleEndian>(client_addr_bytes)?;
            write_unicode_string(buffer, &ext.client_address)?;

            let client_dir_bytes = encode_string_length(&ext.client_dir);
            buffer.write_u16::<LittleEndian>(client_dir_bytes)?;
            write_unicode_string(buffer, &ext.client_dir)?;

            ext.client_time_zone.encode(buffer)?;

            buffer.write_u32::<LittleEndian>(ext.client_session_id)?;
            buffer.write_u32::<LittleEndian>(ext.performance_flags.bits())?;

            // cbAutoReconnectLen (0 - no auto reconnect cookie)
            buffer.write_u16::<LittleEndian>(0)?;
        }

        Ok(())
    }

    fn decode(buffer: &mut dyn Read) -> Result<Self> {
        let code_page = buffer.read_u32::<LittleEndian>()?;
        let flags_bits = buffer.read_u32::<LittleEndian>()?;
        let flags = ClientInfoFlags::from_bits_truncate(flags_bits);

        // Read string lengths
        let cb_domain = buffer.read_u16::<LittleEndian>()?;
        let cb_user_name = buffer.read_u16::<LittleEndian>()?;
        let cb_password = buffer.read_u16::<LittleEndian>()?;
        let cb_alternate_shell = buffer.read_u16::<LittleEndian>()?;
        let cb_working_dir = buffer.read_u16::<LittleEndian>()?;

        // Read strings
        let domain = read_unicode_string(buffer, cb_domain)?;
        let user_name = read_unicode_string(buffer, cb_user_name)?;
        let password = read_unicode_string(buffer, cb_password)?;
        let alternate_shell = read_unicode_string(buffer, cb_alternate_shell)?;
        let working_dir = read_unicode_string(buffer, cb_working_dir)?;

        // Try to read extended info (may not be present)
        let extended_info = if let Ok(client_address_family) = buffer.read_u16::<LittleEndian>() {
            let cb_client_address = buffer.read_u16::<LittleEndian>()?;
            let client_address = read_unicode_string(buffer, cb_client_address)?;

            let cb_client_dir = buffer.read_u16::<LittleEndian>()?;
            let client_dir = read_unicode_string(buffer, cb_client_dir)?;

            let client_time_zone = TimeZoneInformation::decode(buffer)?;
            let client_session_id = buffer.read_u32::<LittleEndian>()?;
            let performance_flags_bits = buffer.read_u32::<LittleEndian>()?;
            let performance_flags = PerformanceFlags::from_bits_truncate(performance_flags_bits);

            // Skip auto reconnect cookie (if present)
            if let Ok(cb_auto_reconnect) = buffer.read_u16::<LittleEndian>() {
                if cb_auto_reconnect > 0 {
                    let mut cookie = vec![0u8; cb_auto_reconnect as usize];
                    let _ = buffer.read_exact(&mut cookie);
                }
            }

            Some(ExtendedInfo {
                client_address_family,
                client_address,
                client_dir,
                client_time_zone,
                client_session_id,
                performance_flags,
            })
        } else {
            None
        };

        Ok(Self {
            code_page,
            flags,
            domain,
            user_name,
            password,
            alternate_shell,
            working_dir,
            extended_info,
        })
    }

    fn size(&self) -> usize {
        let mut size = 4 + 4 + 2 + 2 + 2 + 2 + 2; // Fixed header fields

        // String lengths (UTF-16LE with null terminator)
        size += (self.domain.len() + 1) * 2;
        size += (self.user_name.len() + 1) * 2;
        size += (self.password.len() + 1) * 2;
        size += (self.alternate_shell.len() + 1) * 2;
        size += (self.working_dir.len() + 1) * 2;

        if let Some(ref ext) = self.extended_info {
            size += 2; // clientAddressFamily
            size += 2 + (ext.client_address.len() + 1) * 2;
            size += 2 + (ext.client_dir.len() + 1) * 2;
            size += TimeZoneInformation::SIZE;
            size += 4; // clientSessionId
            size += 4; // performanceFlags
            size += 2; // cbAutoReconnectLen
        }

        size
    }
}

// Helper functions for Unicode string encoding/decoding

fn encode_string_length(s: &str) -> u16 {
    ((s.len() + 1) * 2) as u16 // UTF-16LE + null terminator
}

fn write_unicode_string(buffer: &mut dyn Write, s: &str) -> Result<()> {
    for ch in s.encode_utf16() {
        buffer.write_u16::<LittleEndian>(ch)?;
    }
    buffer.write_u16::<LittleEndian>(0)?; // null terminator
    Ok(())
}

fn read_unicode_string(buffer: &mut dyn Read, byte_count: u16) -> Result<String> {
    if byte_count == 0 {
        return Ok(String::new());
    }

    let char_count = byte_count / 2;
    let mut chars = Vec::with_capacity(char_count as usize);

    for _ in 0..char_count {
        let ch = buffer.read_u16::<LittleEndian>()?;
        if ch != 0 {
            chars.push(ch);
        }
    }

    String::from_utf16(&chars)
        .map_err(|e| PduError::ParseError(format!("Invalid UTF-16 string: {}", e)))
}

fn encode_unicode_string(s: &str, buffer: &mut [u8], max_chars: usize) -> Result<()> {
    let chars: Vec<u16> = s.encode_utf16().take(max_chars - 1).collect();

    for (i, &ch) in chars.iter().enumerate() {
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
    fn test_client_info_flags() {
        let flags = ClientInfoFlags::MOUSE | ClientInfoFlags::UNICODE;
        assert_eq!(flags.bits(), 0x11);
        assert!(flags.contains(ClientInfoFlags::MOUSE));
        assert!(flags.contains(ClientInfoFlags::UNICODE));
        assert!(!flags.contains(ClientInfoFlags::COMPRESSION));
    }

    #[test]
    fn test_performance_flags() {
        let flags = PerformanceFlags::DISABLE_WALLPAPER | PerformanceFlags::DISABLE_THEMING;
        assert!(flags.contains(PerformanceFlags::DISABLE_WALLPAPER));
        assert!(!flags.contains(PerformanceFlags::ENABLE_FONT_SMOOTHING));
    }

    #[test]
    fn test_time_zone_information() {
        let tz = TimeZoneInformation::utc();
        let mut buffer = Vec::new();
        tz.encode(&mut buffer).unwrap();

        assert_eq!(buffer.len(), TimeZoneInformation::SIZE);

        let mut cursor = Cursor::new(buffer);
        let decoded = TimeZoneInformation::decode(&mut cursor).unwrap();

        assert_eq!(decoded.bias, 0);
    }

    #[test]
    fn test_client_info_pdu_basic() {
        let pdu = ClientInfoPdu::new("testuser".to_string(), "password123".to_string());

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ClientInfoPdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.user_name, "testuser");
        assert_eq!(decoded.password, "password123");
        assert_eq!(decoded.code_page, 0);
        assert!(decoded.flags.contains(ClientInfoFlags::UNICODE));
    }

    #[test]
    fn test_client_info_pdu_with_domain() {
        let pdu = ClientInfoPdu::new("admin".to_string(), "pass".to_string())
            .with_domain("WORKGROUP".to_string());

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ClientInfoPdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.domain, "WORKGROUP");
        assert_eq!(decoded.user_name, "admin");
    }

    #[test]
    fn test_client_info_pdu_with_extended() {
        let ext = ExtendedInfo {
            client_address_family: 2, // AF_INET
            client_address: "192.168.1.100".to_string(),
            client_dir: "C:\\Users\\Test".to_string(),
            client_time_zone: TimeZoneInformation::utc(),
            client_session_id: 0,
            performance_flags: PerformanceFlags::DISABLE_WALLPAPER,
        };

        let pdu = ClientInfoPdu::new("user".to_string(), "pass".to_string())
            .with_extended_info(ext);

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ClientInfoPdu::decode(&mut cursor).unwrap();

        assert!(decoded.extended_info.is_some());
        let decoded_ext = decoded.extended_info.unwrap();
        assert_eq!(decoded_ext.client_address, "192.168.1.100");
        assert_eq!(decoded_ext.client_address_family, 2);
    }

    #[test]
    fn test_unicode_string_helpers() {
        let original = "Hello 안녕하세요";
        let mut buffer = Vec::new();
        write_unicode_string(&mut buffer, original).unwrap();

        let byte_count = buffer.len() as u16;
        let mut cursor = Cursor::new(buffer);
        let decoded = read_unicode_string(&mut cursor, byte_count).unwrap();

        assert_eq!(decoded, original);
    }

    #[test]
    fn test_empty_strings() {
        let pdu = ClientInfoPdu::new(String::new(), String::new());

        let mut buffer = Vec::new();
        pdu.encode(&mut buffer).unwrap();

        let mut cursor = Cursor::new(buffer);
        let decoded = ClientInfoPdu::decode(&mut cursor).unwrap();

        assert_eq!(decoded.user_name, "");
        assert_eq!(decoded.password, "");
    }
}
