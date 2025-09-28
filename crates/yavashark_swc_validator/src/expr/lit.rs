mod regex_data;

use crate::Validator;
use regex_data::{BINARY_PROPERTIES, PROPERTY_VALUE_PAIRS};
use swc_ecma_ast::{Lit, Regex};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ClassToken {
    Atom,
    Property,
    Dash,
}

impl<'a> Validator<'a> {
    pub fn validate_lit(lit: &Lit) -> Result<(), String> {
        match lit {
            Lit::Regex(regex) => Self::validate_regex_lit(regex),
            _ => Ok(()),
        }
    }

    fn validate_regex_lit(regex: &Regex) -> Result<(), String> {
        let pattern: Vec<char> = regex.exp.as_ref().chars().collect();
        let flags = regex.flags.as_ref();
        let unicode_enabled = flags.contains('u') || flags.contains('v');

        let mut idx = 0usize;
        let mut in_class = false;
        let mut first_in_class = false;
        let mut class_tokens: Vec<ClassToken> = Vec::new();

        while idx < pattern.len() {
            let ch = pattern[idx];

            if ch == '\\' {
                idx += 1;
                if idx >= pattern.len() {
                    return Err("Dangling escape in regular expression literal".to_string());
                }

                let next = pattern[idx];
                if next == 'p' || next == 'P' {
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
                    } else if !is_valid_binary_property(name) {
                        return Err(format!("Unknown Unicode binary property {name}"));
                    }

                    if in_class {
                        class_tokens.push(ClassToken::Property);
                        first_in_class = false;
                    }

                    idx += 1; // Skip closing '}'
                    continue;
                }

                if in_class {
                    class_tokens.push(ClassToken::Atom);
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
                        class_tokens.push(ClassToken::Atom);
                        first_in_class = false;
                        idx += 1;
                        continue;
                    }

                    if ch == ']' {
                        class_tokens.push(ClassToken::Atom);
                        first_in_class = false;
                        idx += 1;
                        continue;
                    }
                }

                if ch == ']' {
                    ensure_no_property_ranges(&class_tokens)?;
                    in_class = false;
                    class_tokens.clear();
                    idx += 1;
                    continue;
                }

                if ch == '-' {
                    class_tokens.push(ClassToken::Dash);
                } else {
                    class_tokens.push(ClassToken::Atom);
                }

                first_in_class = false;
                idx += 1;
                continue;
            }

            if ch == '[' {
                in_class = true;
                first_in_class = true;
                class_tokens.clear();
            }

            idx += 1;
        }

        Ok(())
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

fn is_valid_property_with_value(name: &str, value: &str) -> bool {
    PROPERTY_VALUE_PAIRS
        .iter()
        .find(|(prop, _)| *prop == name)
        .map_or(false, |(_, values)| contains(values, value))
}

fn contains(collection: &[&str], value: &str) -> bool {
    collection.binary_search(&value).is_ok()
}
