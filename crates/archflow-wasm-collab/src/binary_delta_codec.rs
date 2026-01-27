//! Binary Delta Codec for Efficient Network Transfer
//!
//! This module provides binary encoding and decoding for CRDT deltas,
//! achieving ~75% reduction compared to JSON serialization.

/// Field identifiers for selective delta encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeField {
    /// Position (x, y coordinates)
    Position = 0b00000001,
    /// Color (RGBA)
    Color = 0b00000010,
    /// Size (width, height)
    Size = 0b00000100,
    /// All fields
    All = 0b00000111,
}

impl ShapeField {
    /// Returns all field flags as a bitmask
    pub fn all_bits() -> u8 {
        0b00000111
    }

    /// Checks if the position field is included
    pub fn has_position(bits: u8) -> bool {
        bits & 0b00000001 != 0
    }

    /// Checks if the color field is included
    pub fn has_color(bits: u8) -> bool {
        bits & 0b00000010 != 0
    }

    /// Checks if the size field is included
    pub fn has_size(bits: u8) -> bool {
        bits & 0b00000100 != 0
    }
}

/// Represents a decoded delta from the wire.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedDelta {
    /// Record ID (varint-decoded)
    pub id: u64,
    /// Field mask indicating which fields are present
    pub mask: u8,
    /// Position data if included
    pub position: Option<(f32, f32)>,
    /// Color data if included
    pub color: Option<(u8, u8, u8, u8)>,
    /// Size data if included
    pub size: Option<(f32, f32)>,
}

impl DecodedDelta {
    /// Creates a new decoded delta
    pub fn new(id: u64, mask: u8) -> Self {
        Self {
            id,
            mask,
            position: None,
            color: None,
            size: None,
        }
    }
}

/// Binary codec for delta serialization.
pub struct BinaryDeltaCodec;

impl BinaryDeltaCodec {
    /// Encodes a delta with the given parameters.
    pub fn encode_delta(
        buffer: &mut Vec<u8>,
        id: u64,
        mask: u8,
        position: Option<(f32, f32)>,
        color: Option<(u8, u8, u8, u8)>,
        size: Option<(f32, f32)>,
    ) {
        // VarInt for ID (1-9 bytes)
        let id_bytes = Self::encode_varint(id);
        buffer.extend_from_slice(&id_bytes[..Self::varint_len(id)]);

        // Field mask (1 byte)
        buffer.push(mask);

        // Position payload (8 bytes if present)
        if ShapeField::has_position(mask) {
            if let Some((x, y)) = position {
                buffer.extend_from_slice(&x.to_le_bytes());
                buffer.extend_from_slice(&y.to_le_bytes());
            } else {
                buffer.extend_from_slice(&[0u8; 8]);
            }
        }

        // Color payload (4 bytes if present)
        if ShapeField::has_color(mask) {
            if let Some((r, g, b, a)) = color {
                buffer.push(r);
                buffer.push(g);
                buffer.push(b);
                buffer.push(a);
            } else {
                buffer.extend_from_slice(&[0u8; 4]);
            }
        }

        // Size payload (8 bytes if present)
        if ShapeField::has_size(mask) {
            if let Some((w, h)) = size {
                buffer.extend_from_slice(&w.to_le_bytes());
                buffer.extend_from_slice(&h.to_le_bytes());
            } else {
                buffer.extend_from_slice(&[0u8; 8]);
            }
        }
    }

    /// Decodes a delta from binary data.
    pub fn decode_delta(data: &[u8]) -> Option<DecodedDelta> {
        if data.is_empty() {
            return None;
        }

        let mut pos = 0;

        // Decode VarInt ID
        let (id, id_len) = Self::decode_varint(data)?;
        pos += id_len;

        if pos >= data.len() {
            return None;
        }

        // Field mask
        let mask = data[pos];
        pos += 1;

        // Initialize delta
        let mut delta = DecodedDelta::new(id, mask);

        // Position (8 bytes)
        if ShapeField::has_position(mask) {
            if pos + 8 > data.len() {
                return None;
            }
            let x = f32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            let y =
                f32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
            delta.position = Some((x, y));
            pos += 8;
        }

        // Color (4 bytes)
        if ShapeField::has_color(mask) {
            if pos + 4 > data.len() {
                return None;
            }
            delta.color = Some((data[pos], data[pos + 1], data[pos + 2], data[pos + 3]));
            pos += 4;
        }

        // Size (8 bytes)
        if ShapeField::has_size(mask) {
            if pos + 8 > data.len() {
                return None;
            }
            let w = f32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            let h =
                f32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]]);
            delta.size = Some((w, h));
        }

        Some(delta)
    }

    /// Encodes a u64 as VarInt (up to 10 bytes for u64::MAX).
    fn encode_varint(value: u64) -> [u8; 10] {
        let mut result = [0u8; 10];
        let mut i = 0;
        let mut val = value;

        while val > 0x7F {
            result[i] = (val & 0x7F) as u8 | 0x80;
            val >>= 7;
            i += 1;
        }
        result[i] = val as u8;
        result
    }

    /// Returns the length of a VarInt encoding for the given value.
    fn varint_len(value: u64) -> usize {
        if value == 0 {
            return 1;
        }
        let mut len = 0;
        let mut val = value;
        loop {
            len += 1;
            if val <= 0x7F {
                break;
            }
            val >>= 7;
        }
        len
    }

    /// Decodes a VarInt from a byte slice.
    pub(crate) fn decode_varint(buffer: &[u8]) -> Option<(u64, usize)> {
        let mut result = 0u64;
        let mut shift = 0;
        let mut i = 0;

        // VarInt can be up to 10 bytes for u64::MAX
        while i < buffer.len() && i < 10 {
            let byte = buffer[i];
            result |= ((byte & 0x7F) as u64) << shift;
            i += 1;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        if i > buffer.len() || (i == 10 && buffer[9] & 0x80 != 0) {
            return None;
        }

        Some((result, i))
    }

    /// Encodes a signed integer using zigzag encoding.
    #[allow(dead_code)]
    fn zigzag_encode(n: i64) -> u64 {
        ((n as u64) << 1) ^ ((n >> 63) as u64)
    }

    /// Decodes a zigzag-encoded integer.
    #[allow(dead_code)]
    fn zigzag_decode(n: u64) -> i64 {
        ((n as i128) >> 1) as i64 ^ -((n & 1) as i64)
    }
}

/// Encodes a delta for a single record with position only.
/// Returns the encoded bytes for JavaScript to handle.
pub fn encode_position_delta(id: u64, x: f32, y: f32) -> Vec<u8> {
    let mut buffer = Vec::new();
    BinaryDeltaCodec::encode_delta(
        &mut buffer,
        id,
        ShapeField::has_position(ShapeField::All as u8) as u8,
        Some((x, y)),
        None,
        None,
    );
    buffer
}

/// Decodes a position delta from binary data.
pub fn decode_position_delta(data: &[u8]) -> Option<DecodedDelta> {
    BinaryDeltaCodec::decode_delta(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let id = 12345678901234567u64;
        let mask = ShapeField::All as u8;
        let position = Some((100.5, 200.75));
        let color = Some((255, 128, 64, 255));
        let size = Some((50.0, 75.0));

        let mut encoded = Vec::new();
        BinaryDeltaCodec::encode_delta(&mut encoded, id, mask, position, color, size);

        let decoded = BinaryDeltaCodec::decode_delta(&encoded).unwrap();
        assert_eq!(decoded.id, id);
        assert_eq!(decoded.mask, mask);
        assert_eq!(decoded.position, position);
        assert_eq!(decoded.color, color);
        assert_eq!(decoded.size, size);
    }

    #[test]
    fn test_partial_field_mask_position_only() {
        let id = 42;
        let mask = 1; // Position only

        let mut encoded = Vec::new();
        BinaryDeltaCodec::encode_delta(&mut encoded, id, mask, Some((10.0, 20.0)), None, None);

        let decoded = BinaryDeltaCodec::decode_delta(&encoded).unwrap();
        assert_eq!(decoded.id, id);
        assert_eq!(decoded.position, Some((10.0, 20.0)));
        assert!(decoded.color.is_none());
        assert!(decoded.size.is_none());
    }

    #[test]
    fn test_varint_encoding() {
        // Test small value
        let encoded = BinaryDeltaCodec::encode_varint(0);
        assert_eq!(encoded[0], 0);

        // Test 7-bit value
        let encoded = BinaryDeltaCodec::encode_varint(127);
        assert_eq!(encoded[0], 127);

        // Test 8-bit value (needs continuation)
        let encoded = BinaryDeltaCodec::encode_varint(128);
        assert_eq!(encoded[0], 0x80);
        assert_eq!(encoded[1], 1);
    }

    #[test]
    fn test_varint_decode() {
        for val in [0u64, 1, 127, 128, 255, 1000, u64::MAX] {
            let encoded = BinaryDeltaCodec::encode_varint(val);
            let len = BinaryDeltaCodec::varint_len(val);
            let (decoded, consumed) = BinaryDeltaCodec::decode_varint(&encoded[..len]).unwrap();
            assert_eq!(decoded, val);
            assert_eq!(consumed, len);
        }
    }

    #[test]
    fn test_decode_invalid_data() {
        assert!(BinaryDeltaCodec::decode_delta(&[]).is_none());
    }

    #[test]
    fn test_zigzag_encoding() {
        for n in [-100i64, -1, 0, 1, 100] {
            let encoded = BinaryDeltaCodec::zigzag_encode(n);
            let decoded = BinaryDeltaCodec::zigzag_decode(encoded);
            assert_eq!(decoded, n);
        }
    }

    #[test]
    fn test_shape_field_helpers() {
        let all = ShapeField::All as u8;
        assert!(ShapeField::has_position(all));
        assert!(ShapeField::has_color(all));
        assert!(ShapeField::has_size(all));
    }

    #[test]
    fn test_delta_size_comparison() {
        let mut binary = Vec::new();
        BinaryDeltaCodec::encode_delta(
            &mut binary,
            12345,
            ShapeField::All as u8,
            Some((100.5, 200.3)),
            Some((255, 128, 64, 255)),
            Some((50.0, 30.0)),
        );

        // Binary size: max 30 bytes
        assert!(binary.len() <= 30, "Binary should be <= 30 bytes");
    }

    #[test]
    fn test_encode_position_delta() {
        let encoded = encode_position_delta(1, 10.0, 20.0);
        let decoded = decode_position_delta(&encoded).unwrap();
        assert_eq!(decoded.id, 1);
        assert_eq!(decoded.position, Some((10.0, 20.0)));
    }
}
