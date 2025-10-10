use crate::error::Error;
use crate::value::IntoValue;
use crate::{NativeFunction, Realm, Res, Value, ValueResult};
use std::fmt::Write;
use std::iter::Peekable;
use std::str::Chars;

#[must_use]
pub fn get_escape(realm: &mut Realm) -> Value {
    NativeFunction::with_len(
        "escape",
        |args, _, realm| {
            if args.len() != 1 {
                return Ok("undefined".into());
            }

            let arg = args[0].to_string(realm)?;

            let mut result = String::with_capacity(arg.len());

            for c in arg.chars() {
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
    .into_value()
}

#[must_use]
pub fn get_encode_uri(realm: &mut Realm) -> Value {
    NativeFunction::with_len(
        "encodeURI",
        |args, _, realm| {
            if args.len() != 1 {
                return Ok("undefined".into());
            }

            let arg = args[0].to_string(realm)?;

            let mut result = String::with_capacity(arg.len());

            for c in arg.chars() {
                if is_ascii_uri(c) {
                    result.push(c);
                    continue;
                }

                let _ = write!(result, "%{:02X}", c as u16);
            }

            Ok(result.into())
        },
        realm,
        1,
    )
    .into_value()
}

#[must_use]
pub fn get_encode_uri_component(realm: &mut Realm) -> Value {
    NativeFunction::with_len(
        "encodeURIComponent",
        |args, _, realm| {
            if args.len() != 1 {
                return Ok("undefined".into());
            }

            let arg = args[0].to_string(realm)?;

            let mut result = String::with_capacity(arg.len());

            for c in arg.chars() {
                if is_ascii_uri_component(c) {
                    result.push(c);
                    continue;
                }

                let _ = write!(result, "%{:02X}", c as u16);
            }

            Ok(result.into())
        },
        realm,
        1,
    )
    .into_value()
}

#[must_use]
pub fn get_unescape(realm: &mut Realm) -> Value {
    NativeFunction::with_len(
        "unescape",
        |args, _, realm| {
            if args.len() != 1 {
                return Ok("undefined".into());
            }

            let arg = args[0].to_string(realm)?;

            let mut result = String::with_capacity(arg.len());

            let mut chars = arg.chars();

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
    .into_value()
}

#[must_use]
pub fn get_decode_uri(realm: &mut Realm) -> Value {
    NativeFunction::with_len(
        "decodeURI",
        |args, _, realm| {
            if args.len() != 1 {
                return Ok("undefined".into());
            }

            let arg = args[0].to_string(realm)?;
            decode_uri_impl(&arg, false, realm)
        },
        realm,
        1,
    )
    .into_value()
}

#[must_use]
pub fn get_decode_uri_component(realm: &mut Realm) -> Value {
    NativeFunction::with_len(
        "decodeURIComponent",
        |args, _, realm| {
            if args.len() != 1 {
                return Ok("undefined".into());
            }

            let arg = args[0].to_string(realm)?;
            decode_uri_impl(&arg, true, realm)
        },
        realm,
        1,
    )
    .into_value()
}

fn decode_uri_impl(input: &str, decode_all: bool, realm: &Realm) -> ValueResult {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let bytes = parse_percent_sequence(&mut chars, realm)?;

            match std::str::from_utf8(&bytes) {
                Ok(s) => {
                    if decode_all || !is_reserved_unescaped(s) {
                        result.push_str(s);
                    } else {
                        result.push('%');
                        for &byte in &bytes {
                            write!(result, "{byte:02X}")?;
                        }
                    }
                }
                Err(_) => {
                    return Err(Error::uri_error("Invalid UTF-8 sequence"));
                }
            }
        } else {
            result.push(c);
        }
    }

    Ok(result.into())
}

fn parse_percent_sequence(chars: &mut Peekable<Chars>, _realm: &Realm) -> Res<Vec<u8>> {
    let mut bytes = Vec::new();

    loop {
        let hex1 = chars
            .next()
            .ok_or_else(|| Error::uri_error("Incomplete percent sequence"))?;
        let hex2 = chars
            .next()
            .ok_or_else(|| Error::uri_error("Incomplete percent sequence"))?;

        let digit1 = hex1
            .to_digit(16)
            .ok_or_else(|| Error::uri_error("Invalid hex digit"))?;
        let digit2 = hex2
            .to_digit(16)
            .ok_or_else(|| Error::uri_error("Invalid hex digit"))?;

        let byte = (digit1 * 16 + digit2) as u8;
        bytes.push(byte);

        if bytes.len() == 1 {
            let expected_length = utf8_sequence_length(byte)?;
            if expected_length == 1 {
                break;
            }

            for _ in 1..expected_length {
                if chars.peek() != Some(&'%') {
                    return Err(Error::uri_error("Invalid UTF-8 sequence"));
                }
                chars.next();

                let hex1 = chars
                    .next()
                    .ok_or_else(|| Error::uri_error("Incomplete percent sequence"))?;
                let hex2 = chars
                    .next()
                    .ok_or_else(|| Error::uri_error("Incomplete percent sequence"))?;

                let digit1 = hex1
                    .to_digit(16)
                    .ok_or_else(|| Error::uri_error("Invalid hex digit"))?;
                let digit2 = hex2
                    .to_digit(16)
                    .ok_or_else(|| Error::uri_error("Invalid hex digit"))?;

                let continuation_byte = (digit1 * 16 + digit2) as u8;

                if (continuation_byte & 0xC0) != 0x80 {
                    return Err(Error::uri_error("Invalid UTF-8 continuation byte"));
                }

                bytes.push(continuation_byte);
            }
            break;
        }
    }

    Ok(bytes)
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

fn is_reserved_unescaped(s: &str) -> bool {
    s.chars().any(|c| {
        matches!(
            c,
            ';' | '/' | '?' | ':' | '@' | '&' | '=' | '+' | '$' | ',' | '#'
        )
    })
}

const fn is_ascii_world(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '*' | '@' | '_' | '+' | '-' | '.' | '/')
}

const fn is_ascii_uri(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '-' | '_'
                | '.'
                | '!'
                | '~'
                | '*'
                | '\''
                | '('
                | ')'
                | ';'
                | '/'
                | '?'
                | ':'
                | '@'
                | '&'
                | '='
                | '+'
                | '$'
                | ','
                | '#'
        )
}

const fn is_ascii_uri_component(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')')
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
