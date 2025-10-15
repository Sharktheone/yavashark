mod regex_data;

use crate::Validator;
use regex_data::{BINARY_PROPERTIES, PROPERTY_VALUE_PAIRS, STRING_PROPERTIES};
use swc_ecma_ast::{Lit, Regex};

const RESERVED_DOUBLE_PUNCTUATORS: &[&str] = &[
    "!!", "##", "$$", "%%", "**", "++", ",,", "..", "::", ";;", "<<", "==", ">>", "??", "@@", "``",
    "~~", "^^", "||",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ClassToken {
    Atom,
    Property,
    Dash,
}

impl<'a> Validator<'a> {
    pub fn validate_lit(&mut self, lit: &Lit) -> Result<(), String> {
        match lit {
            Lit::Regex(regex) => Self::validate_regex_lit(regex),
            Lit::Num(num) => self.validate_num_lit(num),
            _ => Ok(()),
        }
    }

    fn validate_num_lit(&self, num: &swc_ecma_ast::Number) -> Result<(), String> {
        let raw = num.raw.as_ref().map(|a| a.as_ref());

        if let Some(raw_str) = raw {
            if raw_str.starts_with('0') && raw_str.len() >= 2 {
                let chars: Vec<char> = raw_str.chars().collect();
                if chars.len() > 1 {
                    let second = chars[1];

                    if second == 'x'
                        || second == 'X'
                        || second == 'b'
                        || second == 'B'
                        || second == 'o'
                        || second == 'O'
                        || second == '.'
                    {
                        return Ok(());
                    }

                    let mut has_separator = false;
                    let mut all_digits = true;

                    for i in 1..chars.len() {
                        let ch = chars[i];

                        if ch == '.' || ch == 'e' || ch == 'E' {
                            all_digits = false;
                            break;
                        }

                        if ch == '_' {
                            has_separator = true;
                        } else if !ch.is_ascii_digit() {
                            all_digits = false;
                            break;
                        }
                    }

                    if has_separator && all_digits {
                        return Err(format!(
                            "Numeric separators are not allowed in legacy octal-like or non-octal decimal integer literals: {raw_str}",
                        ));
                    }
                }
            }

            if self.in_strict_mode() {
                if raw_str.len() >= 2 && raw_str.starts_with('0') {
                    let Some(second_char) = raw_str.chars().nth(1) else {
                        return Ok(());
                    };

                    if second_char.is_ascii_digit() && second_char != '8' && second_char != '9' {
                        if second_char != '.' {
                            return Err(format!(
                                "Legacy octal literals are not allowed in strict mode: {}",
                                raw_str
                            ));
                        }
                    }

                    if raw_str.starts_with('0') && raw_str.len() >= 2 {
                        let chars: Vec<char> = raw_str.chars().collect();
                        if chars[1].is_ascii_digit() {
                            for i in 1..chars.len() {
                                let ch = chars[i];
                                if ch == '8' || ch == '9' {
                                    let before = &raw_str[..i];
                                    if !before.contains('.')
                                        && !before.contains('e')
                                        && !before.contains('E')
                                    {
                                        return Err(format!(
                                            "Non-octal decimal integer literals are not allowed in strict mode: {}",
                                            raw_str
                                        ));
                                    }
                                }
                                if ch == '.' || ch == 'e' || ch == 'E' {
                                    break;
                                }
                                if !ch.is_ascii_digit() && ch != '_' {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_regex_lit(regex: &Regex) -> Result<(), String> {
        let pattern: Vec<char> = regex.exp.as_ref().chars().collect();
        let flags = regex.flags.as_ref();
        let has_v_flag = flags.contains('v');
        let unicode_enabled = flags.contains('u') || has_v_flag;

        if flags.contains('u') && has_v_flag {
            return Err(
                "Regular expression literals cannot use both the /u and /v flags".to_string(),
            );
        }

        if has_v_flag {
            VFlagValidator::new(&pattern).validate()?;
        }

        let mut idx = 0usize;
        let mut in_class = false;
        let mut first_in_class = false;
        let mut class_tokens: Vec<ClassToken> = Vec::new();
        let track_property_ranges = !has_v_flag;

        while idx < pattern.len() {
            let ch = pattern[idx];

            if ch == '\\' {
                idx += 1;
                if idx >= pattern.len() {
                    return Err("Dangling escape in regular expression literal".to_string());
                }

                let mut next = pattern[idx];

                if next == '\\' {
                    if idx + 1 < pattern.len() {
                        let candidate = pattern[idx + 1];
                        if candidate == 'p' || candidate == 'P' {
                            idx += 1;
                            next = candidate;
                        } else {
                            if in_class {
                                if track_property_ranges {
                                    class_tokens.push(ClassToken::Atom);
                                }
                                first_in_class = false;
                            }

                            idx += 1;
                            continue;
                        }
                    } else {
                        if in_class {
                            if track_property_ranges {
                                class_tokens.push(ClassToken::Atom);
                            }
                            first_in_class = false;
                        }

                        idx += 1;
                        continue;
                    }
                }

                if next == 'p' || next == 'P' {
                    let negated = next == 'P';
                    if !unicode_enabled {
                        return Err(
                            "Unicode property escapes require the /u or /v flag".to_string()
                        );
                    }

                    idx += 1;
                    if idx >= pattern.len() || pattern[idx] != '{' {
                        return Err("Unicode property escape must use braces".to_string());
                    }

                    idx += 1;
                    let mut content = String::new();
                    while idx < pattern.len() && pattern[idx] != '}' {
                        content.push(pattern[idx]);
                        idx += 1;
                    }

                    if idx >= pattern.len() {
                        return Err(
                            "Unicode property escape is missing a closing brace".to_string()
                        );
                    }

                    if content.is_empty() {
                        return Err("Unicode property escape cannot be empty".to_string());
                    }

                    if content.chars().any(|c| c.is_whitespace()) {
                        return Err("Unicode property escape cannot contain whitespace".to_string());
                    }

                    let (name, value) = split_property_content(&content)?;

                    if let Some(value) = value {
                        if !is_valid_property_with_value(name, value) {
                            return Err(format!("Unknown Unicode property {name}={value}"));
                        }
                    } else {
                        if is_string_property(name) {
                            if !has_v_flag {
                                return Err(
                                    "Unicode string properties require the /v flag".to_string()
                                );
                            }

                            if negated {
                                return Err(
                                    "Unicode string properties cannot be negated".to_string()
                                );
                            }
                        }

                        if !is_valid_binary_property(name) {
                            return Err(format!("Unknown Unicode binary property {name}"));
                        }
                    }

                    if in_class {
                        if track_property_ranges {
                            class_tokens.push(ClassToken::Property);
                        }
                        first_in_class = false;
                    }

                    idx += 1; // Skip closing '}'
                    continue;
                }

                if in_class {
                    if track_property_ranges {
                        class_tokens.push(ClassToken::Atom);
                    }
                    first_in_class = false;
                }

                idx += 1;
                continue;
            }

            if in_class {
                if first_in_class {
                    if ch == '^' {
                        first_in_class = false;
                        idx += 1;
                        continue;
                    }

                    if ch == '-' {
                        if track_property_ranges {
                            class_tokens.push(ClassToken::Atom);
                        }
                        first_in_class = false;
                        idx += 1;
                        continue;
                    }

                    if ch == ']' {
                        if track_property_ranges {
                            class_tokens.push(ClassToken::Atom);
                        }
                        first_in_class = false;
                        idx += 1;
                        continue;
                    }
                }

                if ch == ']' {
                    if track_property_ranges {
                        ensure_no_property_ranges(&class_tokens)?;
                    }
                    in_class = false;
                    if track_property_ranges {
                        class_tokens.clear();
                    }
                    idx += 1;
                    continue;
                }

                if track_property_ranges {
                    if ch == '-' {
                        class_tokens.push(ClassToken::Dash);
                    } else {
                        class_tokens.push(ClassToken::Atom);
                    }
                }

                first_in_class = false;
                idx += 1;
                continue;
            }

            if ch == '[' {
                in_class = true;
                first_in_class = true;
                if track_property_ranges {
                    class_tokens.clear();
                }
            }

            idx += 1;
        }

        Ok(())
    }
}

struct VFlagValidator<'a> {
    pattern: &'a [char],
    idx: usize,
}

enum OperandKind {
    Consumed,
    Negation,
}

enum OperatorAction {
    ExpectOperand,
    StayAfterOperand,
}

impl<'a> VFlagValidator<'a> {
    fn new(pattern: &'a [char]) -> Self {
        Self { pattern, idx: 0 }
    }

    fn validate(mut self) -> Result<(), String> {
        while let Some(ch) = self.current() {
            match ch {
                '[' => self.parse_class_set()?,
                '\\' => {
                    self.parse_escape()?;
                }
                _ => {
                    self.idx += 1;
                }
            }
        }

        Ok(())
    }

    fn parse_class_set(&mut self) -> Result<(), String> {
        self.idx += 1; // skip '['

        let mut first = true;
        let mut expect_operand = true;
        let mut consumed_any = false;

        loop {
            let Some(ch) = self.current() else {
                return Err("Unterminated character class".to_string());
            };

            if ch == ']' {
                if !consumed_any {
                    return Err("Character class cannot be empty under the /v flag".to_string());
                }

                if expect_operand {
                    return Err(
                        "Character class set operator is missing a right-hand operand".to_string(),
                    );
                }

                self.idx += 1;
                return Ok(());
            }

            if expect_operand {
                match self.parse_operand(first)? {
                    OperandKind::Consumed => {
                        consumed_any = true;
                        expect_operand = false;
                        first = false;
                    }
                    OperandKind::Negation => {
                        first = false;
                    }
                }
                continue;
            }

            if let Some(action) = self.try_parse_operator()? {
                match action {
                    OperatorAction::ExpectOperand => {
                        expect_operand = true;
                    }
                    OperatorAction::StayAfterOperand => {
                        // range operator keeps us in operand-consumed state
                        expect_operand = false;
                    }
                }
                continue;
            }

            // implicit union: next token is another operand
            expect_operand = true;
        }
    }

    fn parse_operand(&mut self, allow_caret: bool) -> Result<OperandKind, String> {
        let Some(ch) = self.current() else {
            return Err("Unterminated character class".to_string());
        };

        if ch == '^' && allow_caret {
            self.idx += 1;
            return Ok(OperandKind::Negation);
        }

        if ch == '\\' {
            self.parse_escape()?;
            return Ok(OperandKind::Consumed);
        }

        if self.matches_reserved_double_punctuator() {
            let token = self.collect_double_punctuator();
            return Err(format!(
                "Reserved punctuator `{token}` must be escaped in character classes with the /v flag"
            ));
        }

        match ch {
            '[' => {
                self.parse_class_set()?;
                return Ok(OperandKind::Consumed);
            }
            '(' => {
                return Err(
                    "Parenthesized character class expressions require nested set syntax under the /v flag".
                        to_string(),
                );
            }
            ')' | '}' | '{' | '|' | '/' => {
                return Err(format!(
                    "Character `{ch}` must be escaped inside Unicode sets with the /v flag"
                ));
            }
            '&' => {
                if self.peek_char(1) == Some('&') {
                    return Err("Set operator `&&` is missing a left-hand operand".to_string());
                }
            }
            '-' => {
                if allow_caret {
                    return Err("'-' cannot begin a set element under the /v flag; escape it or rearrange the expression".to_string());
                }

                self.idx += 1;
                return Ok(OperandKind::Consumed);
            }
            _ => {}
        }

        self.idx += 1;
        Ok(OperandKind::Consumed)
    }

    fn try_parse_operator(&mut self) -> Result<Option<OperatorAction>, String> {
        let Some(ch) = self.current() else {
            return Ok(None);
        };

        match ch {
            '&' if self.peek_char(1) == Some('&') => {
                self.idx += 2;
                return Ok(Some(OperatorAction::ExpectOperand));
            }
            '-' => {
                if self.peek_char(1) == Some('-') {
                    self.idx += 2;
                    return Ok(Some(OperatorAction::ExpectOperand));
                }

                self.idx += 1;
                self.expect_operand_after_range()?;
                return Ok(Some(OperatorAction::StayAfterOperand));
            }
            _ => {}
        }

        if self.matches_reserved_double_punctuator() {
            let token = self.collect_double_punctuator();
            return Err(format!(
                "Reserved punctuator `{token}` must be escaped in character classes with the /v flag"
            ));
        }

        Ok(None)
    }

    fn expect_operand_after_range(&mut self) -> Result<(), String> {
        if self.current() == Some(']') {
            return Err(
                "Range in character class requires a right-hand operand before the closing bracket"
                    .to_string(),
            );
        }

        let result = self.parse_operand(false)?;
        match result {
            OperandKind::Consumed => Ok(()),
            OperandKind::Negation => Err(
                "Negation marker `^` is only allowed at the start of a character class".to_string(),
            ),
        }
    }

    fn parse_escape(&mut self) -> Result<(), String> {
        self.idx += 1; // skip '\'

        let Some(ch) = self.current() else {
            return Err("Dangling escape in regular expression literal".to_string());
        };

        match ch {
            'p' | 'P' | 'q' | 'Q' => {
                self.idx += 1;
                self.expect_char('{')?;
                self.consume_until('}')?;
                self.idx += 1; // skip '}'
            }
            'u' | 'x' => {
                if self.peek_char(1) == Some('{') {
                    self.idx += 2; // skip char and '{'
                    self.consume_until('}')?;
                    self.idx += 1;
                } else {
                    let required = if ch == 'u' { 4 } else { 2 };
                    self.idx += 1;
                    for _ in 0..required {
                        let Some(hex) = self.current() else {
                            return Err("Incomplete hexadecimal escape".to_string());
                        };
                        if !hex.is_ascii_hexdigit() {
                            return Err("Hexadecimal escape contains non-hex digit".to_string());
                        }
                        self.idx += 1;
                    }
                }
            }
            _ => {
                self.idx += 1;
            }
        }

        Ok(())
    }

    fn expect_char(&mut self, ch: char) -> Result<(), String> {
        match self.current() {
            Some(current) if current == ch => {
                self.idx += 1;
                Ok(())
            }
            _ => Err(format!("Expected `{ch}` in escape sequence")),
        }
    }

    fn consume_until(&mut self, terminator: char) -> Result<(), String> {
        while let Some(ch) = self.current() {
            if ch == terminator {
                return Ok(());
            }
            self.idx += 1;
        }

        Err(format!("Escape sequence is missing closing `{terminator}`"))
    }

    fn matches_reserved_double_punctuator(&self) -> bool {
        RESERVED_DOUBLE_PUNCTUATORS
            .iter()
            .any(|token| self.starts_with(token))
    }

    fn collect_double_punctuator(&self) -> String {
        let mut buf = String::new();
        if let Some(first) = self.peek_char(0) {
            buf.push(first);
        }
        if let Some(second) = self.peek_char(1) {
            buf.push(second);
        }
        buf
    }

    fn starts_with(&self, token: &str) -> bool {
        token
            .chars()
            .enumerate()
            .all(|(offset, expected)| self.peek_char(offset) == Some(expected))
    }

    fn current(&self) -> Option<char> {
        self.pattern.get(self.idx).copied()
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        self.pattern.get(self.idx + offset).copied()
    }
}

fn split_property_content<'a>(content: &'a str) -> Result<(&'a str, Option<&'a str>), String> {
    if let Some(eq_idx) = content.find('=') {
        let (name, rest) = content.split_at(eq_idx);
        let value = &rest[1..];

        if name.is_empty() {
            return Err("Unicode property escape is missing a property name".to_string());
        }

        if value.is_empty() {
            return Err("Unicode property escape is missing a property value".to_string());
        }

        if value.contains('=') {
            return Err("Unicode property escape contains multiple '=' characters".to_string());
        }

        Ok((name, Some(value)))
    } else {
        Ok((content, None))
    }
}

fn ensure_no_property_ranges(tokens: &[ClassToken]) -> Result<(), String> {
    for (idx, token) in tokens.iter().enumerate() {
        if token != &ClassToken::Dash {
            continue;
        }

        if idx == 0 || idx + 1 == tokens.len() {
            continue;
        }

        if tokens[idx - 1] == ClassToken::Property || tokens[idx + 1] == ClassToken::Property {
            return Err(
                "Unicode property escapes cannot participate in character class ranges".to_string(),
            );
        }
    }

    Ok(())
}

fn is_valid_binary_property(name: &str) -> bool {
    contains(BINARY_PROPERTIES, name)
}

fn is_string_property(name: &str) -> bool {
    contains(STRING_PROPERTIES, name)
}

fn is_valid_property_with_value(name: &str, value: &str) -> bool {
    PROPERTY_VALUE_PAIRS
        .iter()
        .find(|(prop, _)| *prop == name)
        .map_or(false, |(_, values)| contains(values, value))
}

fn contains(collection: &[&str], value: &str) -> bool {
    collection.binary_search(&value).is_ok()
}
