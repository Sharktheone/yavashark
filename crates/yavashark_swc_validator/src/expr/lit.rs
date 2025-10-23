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

#[derive(Clone, Copy, PartialEq, Eq)]
enum AssertionType {
    Lookahead,
    NegativeLookahead,
    Lookbehind,
    NegativeLookbehind,
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

        Self::validate_pattern_structure(&pattern, unicode_enabled)?;

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

    fn validate_pattern_structure(pattern: &[char], unicode_enabled: bool) -> Result<(), String> {
        let mut idx = 0;
        let mut in_class = false;
        let mut depth = 0;
        let mut can_quantify = false;
        let mut named_groups = std::collections::HashSet::new();
        let mut group_references = Vec::new();
        let mut assertion_stack: Vec<Option<AssertionType>> = Vec::new(); // Stack to track assertions

        while idx < pattern.len() {
            let ch = pattern[idx];

            if ch == '\\' {
                idx += 1;
                if idx >= pattern.len() {
                    return Err("Dangling escape in regular expression literal".to_string());
                }

                let next = pattern[idx];

                // Validate unicode mode identity escapes
                if unicode_enabled && !in_class {
                    // In unicode mode, only certain characters can be escaped
                    let is_allowed_escape = matches!(
                        next,
                        // Syntax characters
                        '^' | '$' | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '/'
                        // Character class escapes
                        | 'd' | 'D' | 's' | 'S' | 'w' | 'W'
                        // Special escapes
                        | 'b' | 'B' | 'f' | 'n' | 'r' | 't' | 'v'
                        // Hex/Unicode escapes
                        | 'x' | 'u'
                        // Control escape (needs letter after)
                        | 'c'
                        // Backreference
                        | 'k'
                        // Property escapes
                        | 'p' | 'P'
                    ) || next.is_ascii_digit(); // Also allow digits for backreferences like \1, \2, etc.

                    if !is_allowed_escape {
                        return Err(format!(
                            "Invalid escape sequence '\\{}' in unicode mode",
                            next
                        ));
                    }
                }

                // Validate \c escape requires letter
                if next == 'c' {
                    if idx + 1 >= pattern.len() {
                        if unicode_enabled {
                            return Err(
                                "Control escape requires a letter in unicode mode".to_string()
                            );
                        }
                    } else if unicode_enabled {
                        let control_char = pattern[idx + 1];
                        if !control_char.is_ascii_alphabetic() {
                            return Err(format!(
                                "Control escape requires a letter, got '{}' in unicode mode",
                                control_char
                            ));
                        }
                    }
                }

                // Check for named group backreference: \k<name>
                if next == 'k' {
                    idx += 1; // Skip 'k'

                    if idx >= pattern.len() || pattern[idx] != '<' {
                        return Err(
                            "Invalid group backreference: expected '<' after \\k".to_string()
                        );
                    }

                    idx += 1; // Skip '<'
                    let mut ref_name = String::new();

                    // Collect the reference name until '>'
                    while idx < pattern.len() && pattern[idx] != '>' {
                        ref_name.push(pattern[idx]);
                        idx += 1;
                    }

                    if idx >= pattern.len() {
                        return Err("Unterminated group backreference".to_string());
                    }

                    if ref_name.is_empty() {
                        return Err("Group backreference name cannot be empty".to_string());
                    }

                    // Store the reference to validate later (after we've collected all group names)
                    group_references.push(ref_name);

                    idx += 1; // Skip '>'
                    can_quantify = true;
                    continue;
                }

                can_quantify = true;
                idx += 1;
                continue;
            }

            if in_class {
                if ch == ']' {
                    in_class = false;
                    can_quantify = true;
                }
                idx += 1;
                continue;
            }

            match ch {
                '[' => {
                    in_class = true;
                    can_quantify = false;
                    idx += 1;
                    continue;
                }
                '(' => {
                    let mut assertion_type_for_this_group: Option<AssertionType> = None;

                    if idx + 1 < pattern.len() && pattern[idx + 1] == '?' {
                        idx += 2; // Skip '(?'

                        if idx >= pattern.len() {
                            return Err("Incomplete group".to_string());
                        }

                        let next = pattern[idx];

                        // Track assertion types for quantifier validation
                        // Check for lookahead/lookbehind assertions
                        if next == '=' {
                            assertion_type_for_this_group = Some(AssertionType::Lookahead);
                        } else if next == '!' {
                            assertion_type_for_this_group = Some(AssertionType::NegativeLookahead);
                        } else if next == '<' {
                            if idx + 1 < pattern.len() {
                                let lookahead_char = pattern[idx + 1];
                                if lookahead_char == '=' {
                                    assertion_type_for_this_group = Some(AssertionType::Lookbehind);
                                } else if lookahead_char == '!' {
                                    assertion_type_for_this_group =
                                        Some(AssertionType::NegativeLookbehind);
                                }
                            }
                        }

                        // Check for named groups: (?<name>...)
                        if next == '<' {
                            idx += 1; // Skip '<'

                            if idx >= pattern.len() {
                                return Err("Incomplete named group".to_string());
                            }

                            // Check if it's not a lookbehind (which would be (?<=...) or (?<!...))
                            if pattern[idx] != '=' && pattern[idx] != '!' {
                                // This is a named capture group
                                let mut group_name = String::new();

                                // Collect the group name until '>'
                                while idx < pattern.len() && pattern[idx] != '>' {
                                    group_name.push(pattern[idx]);
                                    idx += 1;
                                }

                                if idx >= pattern.len() {
                                    return Err("Unterminated named group".to_string());
                                }

                                // Validate the group name
                                if group_name.is_empty() {
                                    return Err("Named group name cannot be empty".to_string());
                                }

                                // Check if first character is a valid ID_Start
                                let first_char = group_name
                                    .chars()
                                    .next()
                                    .ok_or("Named group name cannot be empty")?;
                                if !is_id_start(first_char) {
                                    return Err(format!(
                                        "Invalid character '{}' at start of group name",
                                        first_char
                                    ));
                                }

                                // Check remaining characters are valid ID_Continue
                                for ch in group_name.chars().skip(1) {
                                    if !is_id_continue(ch) {
                                        return Err(format!(
                                            "Invalid character '{}' in group name",
                                            ch
                                        ));
                                    }
                                }

                                // Check for duplicate group names
                                if !named_groups.insert(group_name.clone()) {
                                    return Err(format!(
                                        "Duplicate capture group name '{}'",
                                        group_name
                                    ));
                                }

                                idx += 1; // Skip '>'

                                // Process group opening now, then continue
                                // (to avoid the idx += 1 at the end which would skip a character)
                                depth += 1;
                                can_quantify = false;
                                assertion_stack.push(None); // Named groups are not assertions
                                continue;
                            }
                        } else if next != ':' && next != '=' && next != '!' {
                            // This might be a modifiers group
                            let start = idx;
                            let mut add_flags = String::new();
                            let mut remove_flags = String::new();
                            let mut in_remove = false;
                            let mut has_colon = false;

                            while idx < pattern.len() {
                                let flag_ch = pattern[idx];

                                if flag_ch == ':' {
                                    has_colon = true;
                                    break;
                                }

                                if flag_ch == ')' {
                                    break;
                                }

                                if flag_ch == '-' {
                                    if in_remove {
                                        return Err("Multiple '-' in regexp modifiers".to_string());
                                    }
                                    in_remove = true;
                                    idx += 1;
                                    continue;
                                }

                                // Only i, m, s are allowed in modifiers (not g, d, u, y, v)
                                if !matches!(flag_ch, 'i' | 'm' | 's') {
                                    return Err(format!(
                                        "Invalid flag '{}' in regexp modifiers (only i, m, s are allowed)",
                                        flag_ch
                                    ));
                                }

                                if in_remove {
                                    remove_flags.push(flag_ch);
                                } else {
                                    add_flags.push(flag_ch);
                                }

                                idx += 1;
                            }

                            if start != idx {
                                if add_flags.is_empty() && remove_flags.is_empty() {
                                    return Err(
                                        "At least one RegularExpressionFlags must be present in modifiers"
                                            .to_string(),
                                    );
                                }

                                for flag in add_flags.chars() {
                                    if remove_flags.contains(flag) {
                                        return Err(format!(
                                            "Flag '{}' appears in both add and remove sets",
                                            flag
                                        ));
                                    }
                                }

                                let mut seen = std::collections::HashSet::new();
                                for flag in add_flags.chars().chain(remove_flags.chars()) {
                                    if !seen.insert(flag) {
                                        return Err(format!(
                                            "Duplicate flag '{}' in modifiers",
                                            flag
                                        ));
                                    }
                                }

                                if !has_colon && !remove_flags.is_empty() {
                                    return Err(
                                        "Arithmetic modifiers require ':' before the pattern"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                    }
                    depth += 1;
                    can_quantify = false;
                    assertion_stack.push(assertion_type_for_this_group);
                    idx += 1;
                    continue;
                }
                ')' => {
                    if depth == 0 {
                        return Err("Unmatched closing parenthesis".to_string());
                    }
                    depth -= 1;

                    // Pop the assertion type from the stack
                    let was_assertion = assertion_stack.pop().flatten();

                    // If this was an assertion, we can't quantify it
                    if was_assertion.is_some() {
                        can_quantify = false; // Will be checked by quantifier
                    } else {
                        can_quantify = true;
                    }

                    idx += 1;
                    continue;
                }
                '?' | '*' | '+' => {
                    // Check if the last thing was an assertion
                    // We need to check if we just closed a group that was an assertion
                    if !can_quantify {
                        // This could be because we just closed an assertion
                        // Check the pattern to see if the previous char was ')'
                        if idx > 0 && pattern[idx - 1] == ')' {
                            return Err("Assertions cannot be quantified".to_string());
                        }
                        return Err(format!("Nothing to repeat at position {}", idx));
                    }
                    can_quantify = false;
                    idx += 1;
                    continue;
                }
                '{' => {
                    if pattern.get(idx + 1).map_or(false, |c| c.is_ascii_digit()) {
                        if !can_quantify {
                            if idx > 0 && pattern[idx - 1] == ')' {
                                return Err("Assertions cannot be quantified".to_string());
                            }
                            return Err(format!("Nothing to repeat at position {}", idx));
                        }
                        can_quantify = false;
                    } else {
                        can_quantify = true;
                    }
                    idx += 1;
                    continue;
                }
                '|' => {
                    can_quantify = false;
                    idx += 1;
                    continue;
                }
                '^' | '$' => {
                    can_quantify = false;
                    idx += 1;
                    continue;
                }
                _ => {
                    can_quantify = true;
                    idx += 1;
                    continue;
                }
            }
        }

        if depth != 0 {
            return Err("Unclosed group in regular expression".to_string());
        }

        if in_class {
            return Err("Unterminated character class".to_string());
        }

        // Validate that all group references point to existing groups
        for ref_name in group_references {
            if !named_groups.contains(&ref_name) {
                return Err(format!(
                    "Invalid group backreference: group '{}' does not exist",
                    ref_name
                ));
            }
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

// Helper function to check if a character is valid as the start of an identifier
fn is_id_start(ch: char) -> bool {
    // ID_Start includes:
    // - Letters (Unicode categories Lu, Ll, Lt, Lm, Lo, Nl)
    // - $ and _
    ch == '$' || ch == '_' || ch.is_alphabetic() || matches!(ch, '\u{200C}' | '\u{200D}') // ZWNJ and ZWJ
}

// Helper function to check if a character is valid in an identifier (after the first char)
fn is_id_continue(ch: char) -> bool {
    // ID_Continue includes ID_Start plus:
    // - Digits
    // - Combining marks (Mn, Mc)
    // - Connector punctuation (Pc)
    is_id_start(ch)
        || ch.is_ascii_digit()
        || ch.is_numeric()
        || matches!(ch, '\u{200C}' | '\u{200D}') // ZWNJ and ZWJ
}
