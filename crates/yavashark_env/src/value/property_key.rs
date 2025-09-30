use std::fmt::Display;
use indexmap::Equivalent;
use yavashark_string::{ToYSString, YSString};
use crate::value::{fmt_num, Realm, Symbol, Value};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyKey {
    String(YSString),
    Symbol(Symbol),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BorrowedPropertyKey<'a> {
    String(&'a str),
    Symbol(&'a Symbol),
}

impl Equivalent<PropertyKey> for BorrowedPropertyKey<'_> {
    fn equivalent(&self, other: &PropertyKey) -> bool {
        match (self, other) {
            (Self::String(s), PropertyKey::String(o)) => *s == o.as_str(),
            (Self::Symbol(s), PropertyKey::Symbol(o)) => *s == o,
            _ => false,
        }
    }
}
impl Display for PropertyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Symbol(s) => write!(f, "Symbol{s})"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InternalPropertyKey {
    String(YSString),
    Symbol(Symbol),
    Index(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BorrowedInternalPropertyKey<'a> {
    String(&'a str),
    Symbol(&'a Symbol),
    Index(usize),
}

impl Equivalent<InternalPropertyKey> for BorrowedInternalPropertyKey<'_> {
    fn equivalent(&self, other: &InternalPropertyKey) -> bool {
        match (self, other) {
            (Self::String(s), InternalPropertyKey::String(o)) => *s == o.as_str(),
            (Self::Symbol(s), InternalPropertyKey::Symbol(o)) => *s == o,
            (Self::Index(i), InternalPropertyKey::Index(o)) => *i == *o,
            _ => false,
        }
    }
}

impl Display for InternalPropertyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::Symbol(s) => write!(f, "Symbol{s})"),
            Self::Index(i) => write!(f, "[{i}]"),
        }
    }
}

impl<R: Realm> From<Value<R>> for PropertyKey {
    fn from(value: Value<R>) -> Self {
        match value {
            Value::String(s) => Self::String(s),
            Value::Symbol(s) => Self::Symbol(s),
            Value::Null => Self::String("null".into()),
            Value::Undefined => Self::String("undefined".into()),
            Value::Number(n) => Self::String(n.to_string().into()),
            Value::Boolean(b) => Self::String(b.to_string().into()),
            Value::BigInt(b) => Self::String(b.to_string().into()),
            Value::Object(obj) => Self::String(obj.to_ys_string()),
        }
    }
}

impl<R: Realm> From<PropertyKey> for Value<R> {
    fn from(key: PropertyKey) -> Self {
        match key {
            PropertyKey::String(s) => Self::String(s),
            PropertyKey::Symbol(s) => Self::Symbol(s),
        }
    }
}

impl<R: Realm> From<Value<R>> for InternalPropertyKey {
    fn from(value: Value<R>) -> Self {
        match value {
            Value::String(s) => {
                s.parse::<usize>()
                    .map_or_else(|_| Self::String(s), Self::Index)
                //TODO: this is a hack, we should not parse strings to usize
            }
            Value::Symbol(s) => Self::Symbol(s),
            Value::Null => Self::String("null".into()),
            Value::Undefined => Self::String("undefined".into()),
            Value::Number(n) => {
                if !n.is_nan() && !n.is_infinite() && n.fract() == 0.0 && n.is_sign_positive() {
                    Self::Index(n as usize)
                } else {
                    Self::String(fmt_num(n))
                }
            }
            Value::Boolean(b) => Self::String(b.to_string().into()),
            Value::BigInt(b) => Self::String(b.to_string().into()),
            Value::Object(obj) => Self::String(obj.to_ys_string()),
        }
    }
}

impl<R: Realm> From<InternalPropertyKey> for Value<R> {
    fn from(key: InternalPropertyKey) -> Self {
        match key {
            InternalPropertyKey::String(s) => Self::String(s),
            InternalPropertyKey::Symbol(s) => Self::Symbol(s),
            InternalPropertyKey::Index(i) => Self::Number(i as f64),
        }
    }
}

impl From<InternalPropertyKey> for PropertyKey {
    fn from(key: InternalPropertyKey) -> Self {
        match key {
            InternalPropertyKey::String(s) => Self::String(s),
            InternalPropertyKey::Symbol(s) => Self::Symbol(s),
            InternalPropertyKey::Index(i) => Self::String(i.to_string().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_equivalent_string_keys_have_same_hash() {
        let property_key = PropertyKey::String("test".into());
        let borrowed_key = BorrowedPropertyKey::String("test");

        assert!(borrowed_key.equivalent(&property_key));

        let property_hash = calculate_hash(&property_key);
        let borrowed_hash = calculate_hash(&borrowed_key);
        assert_eq!(property_hash, borrowed_hash);
    }

    #[test]
    fn test_equivalent_symbol_keys_have_same_hash() {
        let symbol = Symbol::from("test_symbol");
        let property_key = PropertyKey::Symbol(symbol.clone());
        let borrowed_key = BorrowedPropertyKey::Symbol(&symbol);

        assert!(borrowed_key.equivalent(&property_key));

        let property_hash = calculate_hash(&property_key);
        let borrowed_hash = calculate_hash(&borrowed_key);
        assert_eq!(property_hash, borrowed_hash);
    }

    #[test]
    fn test_non_equivalent_keys_may_have_different_hashes() {
        let property_key = PropertyKey::String("test".into());
        let borrowed_key = BorrowedPropertyKey::String("different");

        assert!(!borrowed_key.equivalent(&property_key));

        let property_hash = calculate_hash(&property_key);
        let borrowed_hash = calculate_hash(&borrowed_key);

        let _ = (property_hash, borrowed_hash);
    }

    #[test]
    fn test_string_and_symbol_keys_are_not_equivalent() {
        let property_key = PropertyKey::String("test".into());
        let symbol = Symbol::from("test_symbol");
        let borrowed_key = BorrowedPropertyKey::Symbol(&symbol);

        assert!(!borrowed_key.equivalent(&property_key));
    }
}
