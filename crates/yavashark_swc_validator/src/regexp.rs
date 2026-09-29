//! ECMAScript regexp syntax checking. The cursor borrows UTF-8 from SWC;
//! decoded group names and disjunction paths live only in the scratch arena.
#[path = "regexp_data.rs"]
mod data;
use crate::{Result, ValidationError};
use bumpalo::{Bump, collections::Vec as ArenaVec};
use swc_ecma_ast::Regex;
type R<T> = std::result::Result<T, &'static str>;

#[derive(Clone, Copy)]
struct Atom {
    value: Option<u32>,
    strings: bool,
}

impl Atom {
    const SET: Self = Self {
        value: None,
        strings: false,
    };

    const fn ch(c: u32) -> Self {
        Self {
            value: Some(c),
            strings: false,
        }
    }
}

// Branch paths share their immutable ancestry. Capturing a name is O(1)
// storage even in deeply nested groups; only duplicate names compare paths.
struct Branch<'a> {
    id: u32,
    alternative: u32,
    parent: Option<&'a Self>,
}

impl Branch<'_> {
    const fn disjoint(mut left: Option<&Self>, mut right: Option<&Self>) -> bool {
        while let (Some(a), Some(b)) = (left, right) {
            if a.id == b.id {
                if a.alternative != b.alternative {
                    return true;
                }
                left = a.parent;
                right = b.parent;
            } else if a.id > b.id {
                left = a.parent;
            } else {
                right = b.parent;
            }
        }
        false
    }
}

#[derive(Clone, Copy)]
struct Capture<'a> {
    path: &'a Branch<'a>,
    previous: Option<&'a Self>,
}

pub struct Parser<'a> {
    source: &'a str,
    pos: usize,
    unicode: bool,
    sets: bool,
    named: bool,
    captures: u32,
    arena: &'a Bump,
    names: hashbrown::HashMap<&'a str, Capture<'a>, hashbrown::DefaultHashBuilder, &'a Bump>,
    refs: ArenaVec<'a, &'a str>,
    path: Option<&'a Branch<'a>>,
    next_id: u32,
    depth: usize,
}
impl<'a> Parser<'a> {
    pub(super) fn validate<'ast>(r: &'ast Regex, arena: &Bump) -> Result<'ast> {
        let mut flags = 0u16;

        for c in r.flags.bytes() {
            let bit = match c {
                b'd' => 1,
                b'g' => 2,
                b'i' => 4,
                b'm' => 8,
                b's' => 16,
                b'u' => 32,
                b'v' => 64,
                b'y' => 128,
                _ => return Err(ValidationError::new("Invalid regexp flag", r.span)),
            };

            if flags & bit != 0 {
                return Err(ValidationError::new("Duplicate regexp flag", r.span));
            }
            flags |= bit;
        }

        if flags & 0x60 == 0x60 {
            return Err(ValidationError::new("Incompatible regexp flags", r.span));
        }

        let source = r.exp.as_str();

        if source.contains(['\n', '\r', '\u{2028}', '\u{2029}']) {
            return Err(ValidationError::new(
                "Line terminator in regexp literal",
                r.span,
            ));
        }

        let (captures, named) = Self::count_groups(source);
        let mut p = Parser {
            source,
            pos: 0,
            unicode: flags & 0x60 != 0,
            sets: flags & 64 != 0,
            named,
            captures,
            arena,
            names: hashbrown::HashMap::new_in(arena),
            refs: ArenaVec::new_in(arena),
            path: None,
            next_id: 0,
            depth: 0,
        };
        let result = (|| {
            p.disjunction(false)?;

            if p.pos != source.len() {
                return Err("Unmatched regexp parenthesis");
            }

            for name in &p.refs {
                if !p.names.contains_key(name) {
                    return Err("Unknown named regexp reference");
                }
            }

            Ok(())
        })();
        result.map_err(|message| ValidationError::new(message, r.span))
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos).copied()
    }

    fn eat(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn ch(&mut self) -> R<u32> {
        if let Some(byte) = self.peek()
            && byte.is_ascii()
        {
            self.pos += 1;

            return Ok(u32::from(byte));
        }

        let c = self.source[self.pos..]
            .chars()
            .next()
            .ok_or("Unexpected end of regexp")?;
        self.pos += c.len_utf8();

        Ok(c as u32)
    }

    fn disjunction(&mut self, group: bool) -> R<()> {
        self.depth += 1;
        let result = if self.depth.is_multiple_of(16) {
            stacker::maybe_grow(256 * 1024, 2 * 1024 * 1024, || {
                self.disjunction_inner(group)
            })
        } else {
            self.disjunction_inner(group)
        };
        self.depth -= 1;
        result
    }

    fn disjunction_inner(&mut self, group: bool) -> R<()> {
        let id = self.next_id;
        self.next_id += 1;

        let parent = self.path;
        if self.named {
            self.path = Some(self.arena.alloc(Branch {
                id,
                alternative: 0,
                parent,
            }));
        }
        loop {
            match self.peek() {
                None => {
                    if group {
                        return Err("Unclosed regexp group");
                    }
                    break;
                }
                Some(b')') => {
                    if !group {
                        return Err("Unmatched regexp parenthesis");
                    }

                    self.pos += 1;
                    break;
                }
                Some(b'|') => {
                    self.pos += 1;

                    if let Some(branch) = self.path {
                        self.path = Some(self.arena.alloc(Branch {
                            id: branch.id,
                            alternative: branch.alternative + 1,
                            parent: branch.parent,
                        }));
                    }
                    continue;
                }
                _ => {}
            }

            let quantify = match self.peek().ok_or("Unexpected end of regexp")? {
                b'^' | b'$' => {
                    self.pos += 1;
                    false
                }
                b'(' => {
                    self.pos += 1;
                    let mut assertion = false;
                    let mut lookbehind = false;

                    if self.eat(b'?') {
                        if self.eat(b':') {
                        } else if self.eat(b'=') || self.eat(b'!') {
                            assertion = true;
                        } else if self.eat(b'<') {
                            if self.eat(b'=') || self.eat(b'!') {
                                assertion = true;
                                lookbehind = true;
                            } else {
                                let name = self.name()?;
                                self.add_name(name)?;
                            }
                        } else {
                            let mut on = 0;
                            let mut off = 0;
                            let mut minus = false;
                            let mut any = false;
                            while let Some(c) = self.peek() {
                                if c == b':' {
                                    break;
                                }

                                self.pos += 1;

                                if c == b'-' && !minus {
                                    minus = true;
                                    continue;
                                }

                                let bit = match c {
                                    b'i' => 1,
                                    b'm' => 2,
                                    b's' => 4,
                                    _ => return Err("Invalid regexp group"),
                                };
                                any = true;

                                if (on | off) & bit != 0 {
                                    return Err("Duplicate regexp modifier");
                                }

                                if minus { off |= bit } else { on |= bit }
                            }

                            if !any || !self.eat(b':') {
                                return Err("Invalid regexp modifiers");
                            }
                        }
                    }

                    self.disjunction(true)?;
                    !assertion || !self.unicode && !lookbehind
                }
                b'[' => {
                    self.class()?;
                    true
                }
                b'\\' => {
                    self.pos += 1;

                    if matches!(self.peek(), Some(b'b' | b'B')) {
                        self.pos += 1;
                        false
                    } else {
                        self.escape(false)?;
                        true
                    }
                }
                b'*' | b'+' | b'?' => return Err("Nothing to repeat"),
                b'{' => {
                    let before = self.pos;

                    if self.quantifier()?.is_some() {
                        return Err("Nothing to repeat");
                    }

                    self.pos = before;

                    if self.unicode {
                        return Err("Invalid regexp quantifier");
                    }

                    self.pos += 1;
                    true
                }
                b']' | b'}' if self.unicode => return Err("Unescaped regexp syntax character"),
                _ => {
                    // Literal runs need no decoding outside a character class.
                    // Stop only at ASCII grammar characters, hence at UTF-8 boundaries.
                    self.pos += 1;

                    while self
                        .peek()
                        .is_some_and(|byte| !b"^$\\()[]{}*+?|".contains(&byte))
                    {
                        self.pos += 1;
                    }

                    true
                }
            };

            if self.quantifier()?.is_some() {
                if !quantify {
                    return Err("Assertion cannot be quantified");
                }

                self.eat(b'?');
            }
        }

        self.path = parent;

        Ok(())
    }

    fn quantifier(&mut self) -> R<Option<()>> {
        if matches!(self.peek(), Some(b'*' | b'+' | b'?')) {
            self.pos += 1;

            return Ok(Some(()));
        }

        if self.peek() != Some(b'{') {
            return Ok(None);
        }

        let save = self.pos;
        self.pos += 1;
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
        }

        let low = &self.source[start..self.pos];

        if low.is_empty() {
            self.pos = save;

            return Ok(None);
        }

        let mut high = None;

        if self.eat(b',') {
            let start = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.pos += 1;
            }

            if start != self.pos {
                high = Some(&self.source[start..self.pos]);
            }
        }

        if !self.eat(b'}') {
            self.pos = save;

            return Ok(None);
        }

        if let Some(high) = high {
            let a = low.trim_start_matches('0');
            let b = high.trim_start_matches('0');

            if (a.len(), a) > (b.len(), b) {
                return Err("Reversed regexp quantifier");
            }
        }

        Ok(Some(()))
    }

    fn add_name(&mut self, name: &'a str) -> R<()> {
        let path = self.path.ok_or("Missing capture group context")?;

        match self.names.entry(name) {
            hashbrown::hash_map::Entry::Vacant(entry) => {
                entry.insert(Capture {
                    path,
                    previous: None,
                });
            }
            hashbrown::hash_map::Entry::Occupied(mut entry) => {
                let mut previous = Some(entry.get());

                while let Some(capture) = previous {
                    if !Branch::disjoint(Some(capture.path), Some(path)) {
                        return Err("Duplicate regexp capture name");
                    }

                    previous = capture.previous;
                }

                let previous = self.arena.alloc(*entry.get());
                entry.insert(Capture {
                    path,
                    previous: Some(previous),
                });
            }
        }

        Ok(())
    }

    fn name(&mut self) -> R<&'a str> {
        let start = self.pos;
        let mut decoded = None;
        let mut first = true;
        while self.peek() != Some(b'>') {
            let before = self.pos;
            let c = if self.eat(b'\\') {
                if decoded.is_none() {
                    decoded = Some(bumpalo::collections::String::from_str_in(
                        &self.source[start..before],
                        self.arena,
                    ));
                }

                if !self.eat(b'u') {
                    return Err("Invalid regexp group name escape");
                }

                self.unicode_escape()?
            } else {
                self.ch()?
            };
            let ch = char::from_u32(c).ok_or("Invalid regexp group name")?;
            let valid = if first {
                Self::id_start(ch)
            } else {
                Self::id_start(ch)
                    || unicode_id_start::is_id_continue(ch)
                    || matches!(ch, '\u{200c}' | '\u{200d}')
            };

            if !valid {
                return Err("Invalid regexp capture name");
            }
            first = false;

            if let Some(s) = &mut decoded {
                s.push(ch);
            }
        }

        if first {
            return Err("Empty regexp capture name");
        }

        let end = self.pos;
        self.pos += 1;

        Ok(if let Some(s) = decoded {
            s.into_bump_str()
        } else {
            &self.source[start..end]
        })
    }

    fn hex(&mut self, n: usize) -> R<u32> {
        let mut value = 0;

        for _ in 0..n {
            let c = self
                .peek()
                .and_then(|c| (c as char).to_digit(16))
                .ok_or("Invalid hexadecimal regexp escape")?;
            self.pos += 1;
            value = value * 16 + c;
        }

        Ok(value)
    }

    fn unicode_escape(&mut self) -> R<u32> {
        if self.eat(b'{') {
            let mut value = 0u32;
            let start = self.pos;
            while self.peek() != Some(b'}') {
                let c = self
                    .peek()
                    .and_then(|c| (c as char).to_digit(16))
                    .ok_or("Invalid unicode regexp escape")?;
                self.pos += 1;
                value = value
                    .checked_mul(16)
                    .and_then(|v| v.checked_add(c))
                    .filter(|v| *v <= 0x0010_ffff)
                    .ok_or("Unicode regexp escape out of range")?;
            }

            if start == self.pos {
                return Err("Empty unicode regexp escape");
            }

            self.pos += 1;

            Ok(value)
        } else {
            let first = self.hex(4)?;

            if (0xd800..=0xdbff).contains(&first) && self.source[self.pos..].starts_with("\\u") {
                let save = self.pos;
                self.pos += 2;

                if let Ok(low) = self.hex(4)
                    && (0xdc00..=0xdfff).contains(&low)
                {
                    return Ok(0x10000 + ((first - 0xd800) << 10) + low - 0xdc00);
                }

                self.pos = save;
            }

            Ok(first)
        }
    }

    fn escape(&mut self, class: bool) -> R<Atom> {
        let c = self.peek().ok_or("Dangling regexp escape")?;
        self.pos += 1;

        match c {
            b'd' | b'D' | b's' | b'S' | b'w' | b'W' => Ok(Atom::SET),
            b'b' if class => Ok(Atom::ch(8)),
            b'f' => Ok(Atom::ch(12)),
            b'n' => Ok(Atom::ch(10)),
            b'r' => Ok(Atom::ch(13)),
            b't' => Ok(Atom::ch(9)),
            b'v' => Ok(Atom::ch(11)),
            b'c' => {
                if let Some(c) = self.peek()
                    && (c.is_ascii_alphabetic()
                        || !self.unicode && class && (c.is_ascii_digit() || c == b'_'))
                {
                    self.pos += 1;

                    return Ok(Atom::ch(u32::from(c % 32)));
                }

                if self.unicode {
                    Err("Invalid regexp control escape")
                } else {
                    Ok(Atom::ch(u32::from(b'c')))
                }
            }
            b'x' | b'u' => {
                let save = self.pos;
                let result = if c == b'x' {
                    self.hex(2)
                } else if self.unicode {
                    self.unicode_escape()
                } else {
                    self.hex(4)
                };

                match result {
                    Ok(v) => Ok(Atom::ch(v)),
                    Err(e) if self.unicode => Err(e),
                    Err(_) => {
                        self.pos = save;

                        Ok(Atom::ch(u32::from(c)))
                    }
                }
            }
            b'p' | b'P' if self.unicode => {
                if !self.eat(b'{') {
                    return Err("Invalid unicode property escape");
                }

                let start = self.pos;
                while self.peek() != Some(b'}') {
                    if self.peek().is_none() {
                        return Err("Unclosed unicode property escape");
                    }

                    self.pos += 1;
                }

                let content = &self.source[start..self.pos];
                self.pos += 1;
                let strings = data::STRING_PROPERTIES.binary_search(&content).is_ok();
                let valid = if let Some((key, value)) = content.split_once('=') {
                    data::PROPERTY_VALUE_PAIRS
                        .iter()
                        .any(|(k, v)| *k == key && v.binary_search(&value).is_ok())
                } else {
                    data::BINARY_PROPERTIES.binary_search(&content).is_ok()
                };

                if !valid || strings && (!self.sets || c == b'P') {
                    return Err("Unknown unicode property escape");
                }

                Ok(Atom {
                    value: None,
                    strings,
                })
            }
            b'k' if !class && (self.unicode || self.named) => {
                if !self.eat(b'<') {
                    return Err("Invalid named regexp reference");
                }

                let name = self.name()?;
                self.refs.push(name);

                Ok(Atom::SET)
            }
            b'0'..=b'9' => {
                let start = self.pos - 1;

                if c == b'0' && !self.peek().is_some_and(|c| c.is_ascii_digit()) {
                    return Ok(Atom::ch(0));
                }

                if !class && c != b'0' {
                    let mut num = u32::from(c - b'0');
                    while let Some(digit @ b'0'..=b'9') = self.peek() {
                        num = num
                            .saturating_mul(10)
                            .saturating_add(u32::from(digit - b'0'));
                        self.pos += 1;
                    }

                    if num <= self.captures {
                        return Ok(Atom::SET);
                    }

                    self.pos = start + 1;
                }

                if self.unicode {
                    return Err("Invalid decimal regexp escape");
                }

                let mut v = u32::from(c - b'0');

                if c <= b'7' {
                    let max = if c <= b'3' { 2 } else { 1 };

                    for _ in 0..max {
                        if let Some(d @ b'0'..=b'7') = self.peek() {
                            v = v * 8 + u32::from(d - b'0');
                            self.pos += 1;
                        } else {
                            break;
                        }
                    }
                } else {
                    v = u32::from(c);
                }

                Ok(Atom::ch(v))
            }
            b'q' if self.sets && class => {
                if !self.eat(b'{') {
                    return Err("Invalid class string escape");
                }

                let mut len = 0;
                let mut strings = false;
                loop {
                    match self.peek() {
                        None => return Err("Unclosed class string"),
                        Some(b'}') => {
                            self.pos += 1;
                            strings |= len != 1;
                            break;
                        }
                        Some(b'|') => {
                            self.pos += 1;
                            strings |= len != 1;
                            len = 0;
                        }
                        Some(b'\\') => {
                            self.pos += 1;
                            let a = self.escape(true)?;

                            if a.value.is_none() {
                                return Err("Set escape in class string");
                            }
                            len += 1;
                        }
                        Some(_) => {
                            self.set_character()?;
                            len += 1;
                        }
                    }
                }

                Ok(Atom {
                    value: None,
                    strings,
                })
            }
            _ => {
                if self.unicode
                    && !b"^$\\.*+?()[]{}|/".contains(&c)
                    && !(class && c == b'-')
                    && !(self.sets && class && b"&-!#%,:;<=>@`~\"'".contains(&c))
                {
                    return Err("Invalid unicode identity escape");
                }

                if c >= 128 {
                    self.pos -= 1;

                    return Ok(Atom::ch(self.ch()?));
                }

                Ok(Atom::ch(u32::from(c)))
            }
        }
    }

    fn class(&mut self) -> R<Atom> {
        self.depth += 1;
        let result = if self.depth.is_multiple_of(16) {
            stacker::maybe_grow(256 * 1024, 2 * 1024 * 1024, || self.class_inner())
        } else {
            self.class_inner()
        };
        self.depth -= 1;
        result
    }

    fn class_inner(&mut self) -> R<Atom> {
        self.pos += 1;
        let negated = self.eat(b'^');
        let mut strings = false;
        let mut mode = 0;
        let mut operands = 0;
        while self.peek() != Some(b']') {
            if self.peek().is_none() {
                return Err("Unclosed regexp class");
            }

            if self.sets
                && (self.source[self.pos..].starts_with("&&")
                    || self.source[self.pos..].starts_with("--"))
            {
                let next = if self.peek() == Some(b'&') { 1 } else { 2 };

                if operands != 1 || mode != 0 && mode != next {
                    return Err("Invalid class set operator");
                }
                mode = next;
                self.pos += 2;

                if self.peek() == Some(b']') {
                    return Err("Missing class set operand");
                }
                operands = 0;
                continue;
            }

            if self.sets && mode != 0 && operands != 0 {
                return Err("Mixed class set operations");
            }

            let left = self.class_atom()?;
            let mut element_strings = left.strings;

            if self.peek() == Some(b'-')
                && !(self.sets && self.source[self.pos..].starts_with("--"))
                && self.source.as_bytes().get(self.pos + 1) != Some(&b']')
            {
                self.pos += 1;
                let right = self.class_atom()?;

                if mode != 0 {
                    return Err("Range in class set operation");
                }
                match (left.value, right.value) {
                    (Some(l), Some(r)) => {
                        if l > r {
                            return Err("Reversed regexp class range");
                        }
                    }
                    _ => {
                        if self.unicode {
                            return Err("Non-character regexp range endpoint");
                        }
                    }
                }
                element_strings |= right.strings;
            }
            strings = if mode == 1 {
                strings && element_strings
            } else if mode == 2 {
                strings
            } else {
                strings || element_strings
            };
            operands += 1;
        }

        self.pos += 1;

        if self.sets && negated && strings {
            return Err("Negated class containing strings");
        }

        Ok(Atom {
            value: None,
            strings,
        })
    }

    fn class_atom(&mut self) -> R<Atom> {
        match self.peek().ok_or("Unclosed regexp class")? {
            b'\\' => {
                self.pos += 1;
                self.escape(true)
            }
            b'[' if self.sets => self.class(),
            _ if self.sets => Ok(Atom::ch(self.set_character()?)),
            _ => Ok(Atom::ch(self.ch()?)),
        }
    }

    fn set_character(&mut self) -> R<u32> {
        let character = self.peek().ok_or("Missing unicode set character")?;

        if b"()[]{}/-\\|".contains(&character)
            || self.source.as_bytes().get(self.pos + 1) == Some(&character)
                && b"!#$%&*+,.:;<=>?@^`~".contains(&character)
        {
            return Err("Reserved unicode set character");
        }

        self.ch()
    }

    fn id_start(c: char) -> bool {
        matches!(
            c,
            '$' | '_' | '\u{1885}' | '\u{1886}' | '\u{2118}' | '\u{212e}' | '\u{309b}' | '\u{309c}'
        ) || unicode_id_start::is_id_start(c)
    }

    fn count_groups(s: &str) -> (u32, bool) {
        let b = s.as_bytes();
        let mut i = 0;
        let mut class = false;
        let mut count = 0;
        let mut named = false;
        while i < b.len() {
            match b[i] {
                b'\\' => i += 1,
                b'[' => class = true,
                b']' => class = false,
                b'(' if !class => {
                    if b.get(i + 1) != Some(&b'?') {
                        count += 1;
                    } else if b.get(i + 2) == Some(&b'<')
                        && !matches!(b.get(i + 3), Some(b'=' | b'!'))
                    {
                        count += 1;
                        named = true;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        (count, named)
    }
}
