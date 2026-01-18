use crate::error::Error;
use crate::partial_init::Initializer;
use crate::{NativeFunction, ObjectHandle, Realm, Res, ValueResult};
use std::fmt::Write;
use std::str::Chars;

#[must_use]
pub fn get_escape(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "escape",
        |args, _, realm| {
            if args.len() != 1 {
                return Ok("undefined".into());
            }

            let arg = args[0].to_string(realm)?;

            let mut result = String::with_capacity(arg.len());

            for c in arg.as_str_lossy().chars() {
                // is uTF-16

                if is_ascii_world(c) {
                    result.push(c);
                    continue;
                }

                if c.len_utf16() == 1 {
                    let _ = write!(result, "%{:02X}", c as u16);
                } else {
                    let _ = write!(result, "%u{:04X}", c as u32);
                }
            }

            Ok(result.into())
        },
        realm,
        1,
    )
}

pub struct Escape;

impl Initializer<ObjectHandle> for Escape {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_escape(realm))
    }
}

#[must_use]
pub fn get_encode_uri(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "encodeURI",
        |args, _, realm| {
            let arg = if args.is_empty() {
                "undefined".into()
            } else {
                args[0].to_string(realm)?
            };

            encode_impl(&arg.as_str_lossy(), true)
        },
        realm,
        1,
    )
}

pub struct EncodeURI;

impl Initializer<ObjectHandle> for EncodeURI {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_encode_uri(realm))
    }
}

#[must_use]
pub fn get_encode_uri_component(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "encodeURIComponent",
        |args, _, realm| {
            let arg = if args.is_empty() {
                "undefined".into()
            } else {
                args[0].to_string(realm)?
            };

            encode_impl(&arg.as_str_lossy(), false)
        },
        realm,
        1,
    )
}

pub struct EncodeURIComponent;

impl Initializer<ObjectHandle> for EncodeURIComponent {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_encode_uri_component(realm))
    }
}

#[must_use]
pub fn get_unescape(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "unescape",
        |args, _, realm| {
            if args.len() != 1 {
                return Ok("undefined".into());
            }

            let arg = args[0].to_string(realm)?;

            let mut result = String::with_capacity(arg.len());

            let arg_str = arg.as_str_lossy();
            let mut chars = arg_str.chars();

            while let Some(c) = chars.next() {
                let char = unescape_char(c, &mut chars);
                if let Some(char) = char {
                    result.push(char);
                }
            }

            Ok(result.into())
        },
        realm,
        1,
    )
}

pub struct Unescape;

impl Initializer<ObjectHandle> for Unescape {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_unescape(realm))
    }
}

#[must_use]
pub fn get_decode_uri(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "decodeURI",
        |args, _, realm| {
            let arg = if args.is_empty() {
                "undefined".into()
            } else {
                args[0].to_string(realm)?
            };
            decode_uri_impl(&arg.as_str_lossy(), false)
        },
        realm,
        1,
    )
}

pub struct DecodeURI;

impl Initializer<ObjectHandle> for DecodeURI {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_decode_uri(realm))
    }
}

#[must_use]
pub fn get_decode_uri_component(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "decodeURIComponent",
        |args, _, realm| {
            let arg = if args.is_empty() {
                "undefined".into()
            } else {
                args[0].to_string(realm)?
            };
            decode_uri_impl(&arg.as_str_lossy(), true)
        },
        realm,
        1,
    )
}

pub struct DecodeURIComponent;

impl Initializer<ObjectHandle> for DecodeURIComponent {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_decode_uri_component(realm))
    }
}

fn encode_impl(input: &str, is_uri: bool) -> ValueResult {
    let mut result = String::with_capacity(input.len() * 3);

    // Convert to UTF-16 code units to properly handle surrogates
    let code_units: Vec<u16> = input.encode_utf16().collect();
    let len = code_units.len();
    let mut k = 0;

    while k < len {
        let c = code_units[k];

        // Check if the character is in the unescaped set
        if is_unescaped(c, is_uri) {
            result.push(char::from_u32(c as u32).unwrap_or_default());
            k += 1;
            continue;
        }

        // Check for surrogate pairs
        let code_point = if is_high_surrogate(c) {
            // High surrogate (0xD800-0xDBFF)
            if k + 1 >= len {
                // Unpaired high surrogate at end of string
                return Err(Error::uri_error("URI malformed"));
            }
            let next = code_units[k + 1];
            if !is_low_surrogate(next) {
                // High surrogate not followed by low surrogate
                return Err(Error::uri_error("URI malformed"));
            }
            // Combine surrogate pair into code point
            let high = (c as u32) - 0xD800;
            let low = (next as u32) - 0xDC00;
            let cp = (high << 10) + low + 0x10000;
            k += 2;
            cp
        } else if is_low_surrogate(c) {
            // Unpaired low surrogate
            return Err(Error::uri_error("URI malformed"));
        } else {
            k += 1;
            c as u32
        };

        // Encode the code point to UTF-8 and percent-encode each byte
        let mut buf = [0u8; 4];
        let encoded = char::from_u32(code_point)
            .ok_or_else(|| Error::uri_error("Invalid code point"))?
            .encode_utf8(&mut buf);

        for byte in encoded.as_bytes() {
            write!(result, "%{:02X}", byte)?;
        }
    }

    Ok(result.into())
}

const fn is_unescaped(c: u16, is_uri: bool) -> bool {
    // ASCII alphanumeric
    if (c >= 0x30 && c <= 0x39)    // 0-9
        || (c >= 0x41 && c <= 0x5A)  // A-Z
        || (c >= 0x61 && c <= 0x7A)
    // a-z
    {
        return true;
    }

    // uriMark: - _ . ! ~ * ' ( )
    if matches!(
        c,
        0x2D | 0x5F | 0x2E | 0x21 | 0x7E | 0x2A | 0x27 | 0x28 | 0x29
    ) {
        return true;
    }

    // For encodeURI, also include uriReserved: ; / ? : @ & = + $ , #
    if is_uri
        && matches!(
            c,
            0x3B | 0x2F | 0x3F | 0x3A | 0x40 | 0x26 | 0x3D | 0x2B | 0x24 | 0x2C | 0x23
        )
    {
        return true;
    }

    false
}

/// Check if a code unit is a high surrogate (0xD800-0xDBFF)
const fn is_high_surrogate(c: u16) -> bool {
    c >= 0xD800 && c <= 0xDBFF
}

/// Check if a code unit is a low surrogate (0xDC00-0xDFFF)
const fn is_low_surrogate(c: u16) -> bool {
    c >= 0xDC00 && c <= 0xDFFF
}

fn decode_uri_impl(input: &str, decode_all: bool) -> ValueResult {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut k = 0;

    while k < len {
        let c = chars[k];
        if c == '%' {
            if k + 2 >= len {
                return Err(Error::uri_error("Incomplete percent sequence"));
            }

            k += 1; // skip '%'
            let hex1 = chars[k];
            k += 1;
            let hex2 = chars[k];
            k += 1;

            let digit1 = hex1
                .to_digit(16)
                .ok_or_else(|| Error::uri_error("Invalid hex digit"))?;
            let digit2 = hex2
                .to_digit(16)
                .ok_or_else(|| Error::uri_error("Invalid hex digit"))?;

            let byte = (digit1 * 16 + digit2) as u8;

            // Determine UTF-8 sequence length
            let n = utf8_sequence_length(byte)?;

            if n == 1 {
                // Single byte (ASCII)
                let ascii_char = byte as char;

                if decode_all || !is_reserved_unescaped_char(ascii_char) {
                    result.push(ascii_char);
                } else {
                    // Preserve original escape sequence
                    result.push('%');
                    result.push(hex1);
                    result.push(hex2);
                }
            } else {
                // Multi-byte UTF-8 sequence
                let mut bytes = Vec::with_capacity(n);
                bytes.push(byte);

                // Collect the original escape sequences for preservation
                let mut original_escapes = vec![format!("%{}{}", hex1, hex2)];

                for _ in 1..n {
                    if k >= len || chars[k] != '%' {
                        return Err(Error::uri_error("Invalid UTF-8 sequence"));
                    }
                    k += 1; // skip '%'

                    if k + 1 >= len {
                        return Err(Error::uri_error("Incomplete percent sequence"));
                    }

                    let h1 = chars[k];
                    k += 1;
                    let h2 = chars[k];
                    k += 1;

                    let d1 = h1
                        .to_digit(16)
                        .ok_or_else(|| Error::uri_error("Invalid hex digit"))?;
                    let d2 = h2
                        .to_digit(16)
                        .ok_or_else(|| Error::uri_error("Invalid hex digit"))?;

                    let continuation_byte = (d1 * 16 + d2) as u8;

                    if (continuation_byte & 0xC0) != 0x80 {
                        return Err(Error::uri_error("Invalid UTF-8 continuation byte"));
                    }

                    bytes.push(continuation_byte);
                    original_escapes.push(format!("%{}{}", h1, h2));
                }

                // Validate the UTF-8 sequence
                validate_utf8_sequence(&bytes)?;

                match std::str::from_utf8(&bytes) {
                    Ok(s) => {
                        // For multi-byte sequences, always decode (reserved chars are ASCII only)
                        result.push_str(s);
                    }
                    Err(_) => {
                        return Err(Error::uri_error("Invalid UTF-8 sequence"));
                    }
                }
            }
        } else {
            result.push(c);
            k += 1;
        }
    }

    Ok(result.into())
}

/// Check if a single character is in the reserved set for decodeURI
fn is_reserved_unescaped_char(c: char) -> bool {
    matches!(
        c,
        ';' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | ',' | '#'
    )
}

/// Validate a UTF-8 byte sequence for:
/// - Overlong encodings
/// - Surrogate code points (0xD800-0xDFFF are invalid in UTF-8)
/// - Code points beyond 0x10FFFF
fn validate_utf8_sequence(bytes: &[u8]) -> Res<()> {
    let code_point = match bytes.len() {
        1 => bytes[0] as u32,
        2 => ((bytes[0] as u32 & 0x1F) << 6) | (bytes[1] as u32 & 0x3F),
        3 => {
            ((bytes[0] as u32 & 0x0F) << 12)
                | ((bytes[1] as u32 & 0x3F) << 6)
                | (bytes[2] as u32 & 0x3F)
        }
        4 => {
            ((bytes[0] as u32 & 0x07) << 18)
                | ((bytes[1] as u32 & 0x3F) << 12)
                | ((bytes[2] as u32 & 0x3F) << 6)
                | (bytes[3] as u32 & 0x3F)
        }
        _ => return Err(Error::uri_error("Invalid UTF-8 sequence length")),
    };

    // Check for overlong encoding
    let min_code_point = match bytes.len() {
        1 => 0,
        2 => 0x80,
        3 => 0x800,
        4 => 0x10000,
        _ => unreachable!(),
    };

    if code_point < min_code_point {
        return Err(Error::uri_error("Overlong UTF-8 sequence"));
    }

    // Check for surrogate code points (0xD800-0xDFFF)
    if (0xD800..=0xDFFF).contains(&code_point) {
        return Err(Error::uri_error("Invalid UTF-8: surrogate code point"));
    }

    // Check for code points beyond 0x10FFFF
    if code_point > 0x10FFFF {
        return Err(Error::uri_error("Invalid UTF-8: code point too large"));
    }

    Ok(())
}

fn utf8_sequence_length(first_byte: u8) -> Result<usize, Error> {
    if first_byte < 0x80 {
        Ok(1) // ASCII
    } else if (first_byte & 0xE0) == 0xC0 {
        // Check for overlong encoding (0xC0, 0xC1 are invalid)
        if first_byte < 0xC2 {
            return Err(Error::uri_error("Overlong UTF-8 sequence".to_string()));
        }
        Ok(2) // 110xxxxx
    } else if (first_byte & 0xF0) == 0xE0 {
        Ok(3) // 1110xxxx
    } else if (first_byte & 0xF8) == 0xF0 {
        // Check for invalid start bytes (0xF5-0xFF)
        if first_byte > 0xF4 {
            return Err(Error::uri_error("Invalid UTF-8 start byte".to_string()));
        }
        Ok(4) // 11110xxx
    } else {
        Err(Error::uri_error("Invalid UTF-8 start byte".to_string()))
    }
}

const fn is_ascii_world(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '*' | '@' | '_' | '+' | '-' | '.' | '/')
}

fn unescape_char(c: char, chars: &mut Chars) -> Option<char> {
    //TODO: we should also handle invalid sequences differently => %ZZ will be %ZZ in the final string
    match c {
        '%' => {
            let next = chars.next()?;
            if next == 'u' {
                let code = chars.take(4).collect::<String>();
                let code = u32::from_str_radix(&code, 16).ok()?;
                char::from_u32(code)
            } else {
                let code = next.to_digit(16)? * 16 + chars.next()?.to_digit(16)?;
                char::from_u32(code)
            }
        }
        '\\' => {
            let next = chars.next()?;
            match next {
                'b' => Some('\u{0008}'),
                't' => Some('\u{0009}'),
                'n' => Some('\u{000A}'),
                'v' => Some('\u{000B}'),
                'f' => Some('\u{000C}'),
                'r' => Some('\u{000D}'),
                'x' => {
                    let code = chars.take(2).collect::<String>();
                    let code = u32::from_str_radix(&code, 16).ok()?;
                    char::from_u32(code)
                }
                'u' => {
                    let code = chars.take(4).collect::<String>();
                    let code = u32::from_str_radix(&code, 16).ok()?;
                    char::from_u32(code)
                }
                _ => Some(next),
            }
        }
        _ => Some(c),
    }
}
