use crate::{NativeFunction, Realm, Value};
use std::fmt::Write;
use yavashark_value::IntoValue;

#[must_use]
pub fn get_escape(realm: &Realm) -> Value {
    NativeFunction::new(
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
    )
    .into_value()
}

#[must_use]
pub fn get_encode_uri(realm: &Realm) -> Value {
    NativeFunction::new(
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
    )
    .into_value()
}

#[must_use]
pub fn get_encode_uri_component(realm: &Realm) -> Value {
    NativeFunction::new(
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
    )
    .into_value()
}

#[must_use]
pub fn get_unescape(realm: &Realm) -> Value {
    NativeFunction::new(
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
    )
    .into_value()
}

#[must_use]
pub fn get_decode_uri(realm: &Realm) -> Value {
    NativeFunction::new(
        "decodeURI",
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
    )
    .into_value()
}

#[must_use]
pub fn get_decode_uri_component(realm: &Realm) -> Value {
    NativeFunction::new(
        "decodeURIComponent",
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
    )
    .into_value()
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

fn unescape_char(c: char, chars: &mut std::str::Chars) -> Option<char> {
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

// fn decode_uri_char(c: char, result: &mut String, chars: &mut std::str::Chars) -> Option<()> {
//
//
// }
