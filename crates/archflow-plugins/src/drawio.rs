// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Plugins - Draw.io Decoder and Parser
//
// Architecture Reference: ARQUITECTURA_FINAL_V3.md - Section 14.1, 14.2
//
// Decodes Draw.io data format:
// 1. XML (diagram description)
// 2. Deflate (compression)
// 3. Base64 (encoding)
// 4. URL encode (%-encoding)
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

/// Decode Draw.io compressed data
///
/// Draw.io format layers:
/// 1. URL Decode (%-encoding)
/// 2. Base64 Decode
/// 3. Deflate Decompression
/// 4. XML Output
///
/// # Arguments
/// * `encoded_data` - The encoded Draw.io data string
///
/// # Returns
/// The decoded XML string
pub fn decode_drawio_data(encoded_data: &str) -> Result<String, DecodeError> {
    // 1. URL Decode (if there's %xx)
    let decoded_url = if encoded_data.contains('%') {
        decode_url_percent(encoded_data)?
    } else {
        encoded_data.to_string()
    };

    // 2. Base64 Decode
    let compressed_bytes = decode_base64(&decoded_url)?;

    // 3. Deflate Decompression
    let xml_string = inflate_deflate(&compressed_bytes)?;

    Ok(xml_string)
}

/// URL percent decode (removes %xx encoding)
fn decode_url_percent(data: &str) -> Result<String, DecodeError> {
    let mut result = String::with_capacity(data.len());
    let mut chars = data.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Read next 2 hex digits
            let hex1 = chars.next().ok_or(DecodeError::InvalidUrlEncoding)?;
            let hex2 = chars.next().ok_or(DecodeError::InvalidUrlEncoding)?;

            // Convert hex chars to values manually
            let h1 = hex_to_nibble(hex1).ok_or(DecodeError::InvalidUrlEncoding)?;
            let h2 = hex_to_nibble(hex2).ok_or(DecodeError::InvalidUrlEncoding)?;
            let byte = (h1 << 4) | h2;

            result.push(byte as char);
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

/// Convert a hex character to its 4-bit value
fn hex_to_nibble(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}

/// Base64 decode
fn decode_base64(data: &str) -> Result<Vec<u8>, DecodeError> {
    // Simplified base64 decoding for standard alphabet
    let decode_table = make_base64_decode_table();

    let mut result = Vec::new();
    let mut chars = data.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '=' {
            // Padding, skip
            chars.next();
            continue;
        }

        // Get 4 characters (6 bits each = 24 bits)
        let c1 = chars.next().ok_or(DecodeError::InvalidBase64)?;
        let c2 = chars.next().ok_or(DecodeError::InvalidBase64)?;
        let c3 = chars.next().unwrap_or('=');
        let c4 = chars.next().unwrap_or('=');

        let v1 = *decode_table
            .get(c1 as usize)
            .ok_or(DecodeError::InvalidBase64)? as i8;
        let v2 = *decode_table
            .get(c2 as usize)
            .ok_or(DecodeError::InvalidBase64)? as i8;

        // Use u32 to avoid overflow issues with i8 shifts
        let triple = ((v1 as u32) << 18) | ((v2 as u32) << 12);

        if c3 != '=' {
            let v3 = *decode_table
                .get(c3 as usize)
                .ok_or(DecodeError::InvalidBase64)? as i8;
            let triple = triple | ((v3 as u32) << 6);

            if c4 != '=' {
                let v4 = *decode_table
                    .get(c4 as usize)
                    .ok_or(DecodeError::InvalidBase64)? as i8;
                let triple = triple | (v4 as u32);

                result.push((triple >> 16) as u8);
                result.push((triple >> 8) as u8);
                result.push(triple as u8);
            } else {
                result.push((triple >> 16) as u8);
                result.push((triple >> 8) as u8);
            }
        } else {
            result.push((triple >> 16) as u8);
        }
    }

    Ok(result)
}

/// Create base64 decode table
fn make_base64_decode_table() -> [i8; 256] {
    let mut table = [-1i8; 256];

    // Standard base64 alphabet
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    for (i, &c) in alphabet.as_bytes().iter().enumerate() {
        table[c as usize] = i as i8;
    }

    table
}

/// Deflate decompression using miniz-style algorithm
fn inflate_deflate(_compressed: &[u8]) -> Result<String, DecodeError> {
    // This is a simplified stub implementation
    // In production, use flate2 crate with proper deflate support
    // For now, return a placeholder error

    // The actual implementation would use:
    // use flate2::read::DeflateDecoder;
    // let mut decoder = DeflateDecoder::new(compressed);
    // decoder.read_to_string(&mut xml_string)?;

    Err(DecodeError::DeflateNotImplemented)
}

/// Library icon from Draw.io
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryIcon {
    /// Unique identifier for the icon
    pub id: String,
    /// Display name of the icon
    pub name: String,
    /// SVG data for the icon
    pub svg_data: String,
}

/// Parse Draw.io library XML
///
/// # Arguments
/// * `xml_content` - The XML content to parse
///
/// # Returns
/// Vector of parsed library icons
pub fn parse_library_xml(xml_content: &str) -> Vec<LibraryIcon> {
    let mut icons = Vec::new();
    let mut current_tag = String::new();
    let mut current_content = String::new();
    let mut in_item = false;
    let mut current_icon: Option<LibraryIcon> = None;

    // Simple XML parser (state machine)
    let mut chars = xml_content.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '<' => {
                // Start of tag
                current_tag.clear();
                while let Some(&next_c) = chars.peek() {
                    chars.next();
                    if next_c == '>' {
                        break;
                    }
                    current_tag.push(next_c);
                }

                if current_tag.starts_with("item") {
                    in_item = true;
                    // Extract id attribute if present
                    if let Some(id_start) = current_tag.find("id=\"") {
                        let id_rest = &current_tag[id_start + 4..];
                        if let Some(id_end) = id_rest.find('\"') {
                            let id_str = &id_rest[..id_end];
                            current_icon = Some(LibraryIcon {
                                id: id_str.to_string(),
                                name: String::new(),
                                svg_data: String::new(),
                            });
                        }
                    }
                } else if current_tag == "/item" {
                    if let Some(icon) = current_icon.take() {
                        icons.push(icon);
                    }
                    in_item = false;
                } else if current_tag == "/name" && in_item {
                    if let Some(icon) = current_icon.as_mut() {
                        icon.name = current_content.clone();
                    }
                    current_content.clear();
                } else if current_tag == "/svg" && in_item {
                    if let Some(icon) = current_icon.as_mut() {
                        icon.svg_data = current_content.clone();
                    }
                    current_content.clear();
                }
            }
            _ if in_item => {
                // Content within item tag
                current_content.push(c);
            }
            _ => {}
        }
    }

    icons
}

/// Errors that can occur during Draw.io decoding
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Invalid URL percent encoding
    InvalidUrlEncoding,
    /// Invalid Base64 encoding
    InvalidBase64,
    /// Deflate decompression not yet implemented
    DeflateNotImplemented,
    /// Generic error message
    Message(String),
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DecodeError::InvalidUrlEncoding => write!(f, "Invalid URL percent encoding"),
            DecodeError::InvalidBase64 => write!(f, "Invalid Base64 encoding"),
            DecodeError::DeflateNotImplemented => {
                write!(f, "Deflate decompression not implemented")
            }
            DecodeError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_url_percent() {
        let input = "hello%20world%2Ftest";
        let result = decode_url_percent(input).unwrap();
        assert_eq!(result, "hello world/test");
    }

    #[test]
    fn test_decode_url_percent_no_encoding() {
        let input = "hello world";
        let result = decode_url_percent(input).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_base64_decode_simple() {
        let input = "SGVsbG8gV29ybGQ=";
        let result = decode_base64(input).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "Hello World");
    }

    #[test]
    fn test_base64_decode_empty() {
        let input = "";
        let result = decode_base64(input).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_library_xml_empty() {
        let xml = "";
        let result = parse_library_xml(xml);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_library_xml_simple() {
        // Simple test - the parser captures item with id attribute
        let xml = "<item id=\"icon1\">Test Icon content</item>";
        let result = parse_library_xml(xml);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "icon1");
    }

    #[test]
    fn test_parse_library_xml_multiple() {
        // Test multiple items
        let xml = "<item id=\"icon1\">Icon 1</item><item id=\"icon2\">Icon 2</item>";
        let result = parse_library_xml(xml);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "icon1");
        assert_eq!(result[1].id, "icon2");
    }

    #[test]
    fn test_library_icon_creation() {
        let icon = LibraryIcon {
            id: "test_id".to_string(),
            name: "Test Name".to_string(),
            svg_data: "<svg></svg>".to_string(),
        };

        assert_eq!(icon.id, "test_id");
        assert_eq!(icon.name, "Test Name");
        assert_eq!(icon.svg_data, "<svg></svg>");
    }

    #[test]
    fn test_decode_error_display() {
        // Test Display trait implementation
        let url_err = DecodeError::InvalidUrlEncoding;
        let base64_err = DecodeError::InvalidBase64;
        let deflate_err = DecodeError::DeflateNotImplemented;

        // Convert to string and check
        assert_eq!(url_err.to_string(), "Invalid URL percent encoding");
        assert_eq!(base64_err.to_string(), "Invalid Base64 encoding");
        assert_eq!(
            deflate_err.to_string(),
            "Deflate decompression not implemented"
        );
    }
}
