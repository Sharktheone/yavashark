use crate::value::{fmt_num, Symbol, Value};
use crate::{PrimitiveValue, Realm, Res};
use indexmap::Equivalent;
use std::fmt::Display;
use yavashark_string::YSString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropertyKey {
    String(YSString),
    Symbol(Symbol),
}

#[cfg(target_pointer_width = "64")]
const MAX_INDEX: usize = (1 << 53) - 1;
#[cfg(target_pointer_width = "32")]
const MAX_INDEX: usize = (1 << 31) - 1;

#[cfg(target_pointer_width = "16")]
const MAX_INDEX: usize = (1 << 15) - 1;

impl PropertyKey {
    pub fn as_str(&self) -> &str {
        match self {
            Self::String(s) => s.as_str(),
            Self::Symbol(s) => s.as_str(),
        }
    }

    pub fn from_static(s: &'static str) -> Self {
        Self::String(YSString::new_static(s))
    }

    pub fn from_symbol(s: Symbol) -> Self {
        Self::Symbol(s)
    }

    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }
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

// impl From<Value> for PropertyKey {
//     fn from(value: Value) -> Self {
//         match value {
//             Value::String(s) => Self::String(s),
//             Value::Symbol(s) => Self::Symbol(s),
//             Value::Null => Self::String("null".into()),
//             Value::Undefined => Self::String("undefined".into()),
//             Value::Number(n) => Self::String(n.to_string().into()),
//             Value::Boolean(b) => Self::String(b.to_string().into()),
//             Value::BigInt(b) => Self::String(b.to_string().into()),
//             Value::Object(obj) => Self::String(obj.to_ys_string()),
//         }
//     }
// }

impl From<usize> for PropertyKey {
    fn from(index: usize) -> Self {
        Self::String(index.to_string().into())
    }
}

impl From<PropertyKey> for Value {
    fn from(key: PropertyKey) -> Self {
        match key {
            PropertyKey::String(s) => Self::String(s),
            PropertyKey::Symbol(s) => Self::Symbol(s),
        }
    }
}

// impl From<Value> for InternalPropertyKey {
//     fn from(value: Value) -> Self {
//         match value {
//             Value::String(s) => {
//                 s.parse::<usize>()
//                     .map_or_else(|_| Self::String(s), Self::Index)
//                 //TODO: this is a hack, we should not parse strings to usize
//             }
//             Value::Symbol(s) => Self::Symbol(s),
//             Value::Null => Self::String("null".into()),
//             Value::Undefined => Self::String("undefined".into()),
//             Value::Number(n) => {
//                 if !n.is_nan() && !n.is_infinite() && n.fract() == 0.0 && n.is_sign_positive() {
//                     Self::Index(n as usize)
//                 } else {
//                     Self::String(fmt_num(n))
//                 }
//             }
//             Value::Boolean(b) => Self::String(b.to_string().into()),
//             Value::BigInt(b) => Self::String(b.to_string().into()),
//             Value::Object(obj) => Self::String(obj.to_ys_string()),
//         }
//     }
// }

impl From<PropertyKey> for InternalPropertyKey {
    fn from(key: PropertyKey) -> Self {
        match key {
            PropertyKey::String(s) => {
                s.parse::<usize>()
                    .map_or_else(|_| Self::String(s), Self::Index)
                //TODO: this is a hack, we should not parse strings to usize
            }
            PropertyKey::Symbol(s) => Self::Symbol(s),
        }
    }
}

impl From<InternalPropertyKey> for Value {
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

pub trait IntoPropertyKey: Sized {
    fn into_property_key(self, realm: &mut Realm) -> Res<PropertyKey> {
        self.into_internal_property_key(realm).map(Into::into)
    }
    fn into_internal_property_key(self, realm: &mut Realm) -> Res<InternalPropertyKey>;
}

impl IntoPropertyKey for PropertyKey {
    fn into_property_key(self, _realm: &mut Realm) -> Res<PropertyKey> {
        Ok(self)
    }
    fn into_internal_property_key(self, _realm: &mut Realm) -> Res<InternalPropertyKey> {
        match self {
            Self::String(s) => Ok(s.into()),
            Self::Symbol(s) => Ok(InternalPropertyKey::Symbol(s)),
        }
    }
}

impl IntoPropertyKey for InternalPropertyKey {
    fn into_property_key(self, _realm: &mut Realm) -> Res<PropertyKey> {
        Ok(self.into())
    }
    fn into_internal_property_key(self, _realm: &mut Realm) -> Res<InternalPropertyKey> {
        Ok(self)
    }
}

impl IntoPropertyKey for &'static str {
    fn into_property_key(self, _realm: &mut Realm) -> Res<PropertyKey> {
        Ok(PropertyKey::String(self.into()))
    }
    fn into_internal_property_key(self, _realm: &mut Realm) -> Res<InternalPropertyKey> {
        Ok(InternalPropertyKey::String(self.into()))
    }
}

impl IntoPropertyKey for String {
    fn into_property_key(self, _realm: &mut Realm) -> Res<PropertyKey> {
        Ok(PropertyKey::String(self.into()))
    }
    fn into_internal_property_key(self, _realm: &mut Realm) -> Res<InternalPropertyKey> {
        Ok(string_to_internal_property_key(self.into()))
    }
}

impl IntoPropertyKey for YSString {
    fn into_property_key(self, _realm: &mut Realm) -> Res<PropertyKey> {
        Ok(PropertyKey::String(self))
    }
    fn into_internal_property_key(self, _realm: &mut Realm) -> Res<InternalPropertyKey> {
        Ok(string_to_internal_property_key(self))
    }
}

impl IntoPropertyKey for Value {
    fn into_property_key(self, realm: &mut Realm) -> Res<PropertyKey> {
        Ok(match self {
            Self::String(s) => PropertyKey::String(s),
            Self::Symbol(s) => PropertyKey::Symbol(s),
            Self::Null => PropertyKey::String("null".into()),
            Self::Undefined => PropertyKey::String("undefined".into()),
            Self::Number(n) => PropertyKey::String(n.to_string().into()),
            Self::Boolean(b) => PropertyKey::String(b.to_string().into()),
            Self::BigInt(b) => PropertyKey::String(b.to_string().into()),
            Self::Object(obj) => {
                if let Some(primitive) = obj.primitive(realm)? {
                    return primitive.into_property_key(realm);
                }

                PropertyKey::String(obj.to_string(realm)?)
            },
        })
    }
    fn into_internal_property_key(self, realm: &mut Realm) -> Res<InternalPropertyKey> {
        Ok(match self {
            Self::String(s) => string_to_internal_property_key(s),
            Self::Symbol(s) => InternalPropertyKey::Symbol(s),
            Self::Null => InternalPropertyKey::String("null".into()),
            Self::Undefined => InternalPropertyKey::String("undefined".into()),
            Self::Number(n) => {
                if !n.is_nan()
                    && !n.is_infinite()
                    && n.fract() == 0.0
                    && n.is_sign_positive()
                    && n as usize <= MAX_INDEX
                {
                    InternalPropertyKey::Index(n as usize)
                } else {
                    InternalPropertyKey::String(fmt_num(n))
                }
            }
            Self::Boolean(b) => InternalPropertyKey::String(b.to_string().into()),
            Self::BigInt(b) => InternalPropertyKey::String(b.to_string().into()),
            Self::Object(obj) => {
                if let Some(primitive) = obj.primitive(realm)? {
                    return primitive.into_internal_property_key(realm);
                }

                InternalPropertyKey::String(obj.to_string(realm)?)
            }
        })
    }
}

impl IntoPropertyKey for PrimitiveValue {
    fn into_property_key(self, _realm: &mut Realm) -> Res<PropertyKey> {
        Ok(match self {
            Self::String(s) => PropertyKey::String(s),
            Self::Symbol(s) => PropertyKey::Symbol(s),
            Self::Null => PropertyKey::String("null".into()),
            Self::Undefined => PropertyKey::String("undefined".into()),
            Self::Number(n) => PropertyKey::String(n.to_string().into()),
            Self::Boolean(b) => PropertyKey::String(b.to_string().into()),
            Self::BigInt(b) => PropertyKey::String(b.to_string().into()),
        })
    }
    fn into_internal_property_key(self, _realm: &mut Realm) -> Res<InternalPropertyKey> {
        Ok(match self {
            Self::String(s) => string_to_internal_property_key(s),
            Self::Symbol(s) => InternalPropertyKey::Symbol(s),
            Self::Null => InternalPropertyKey::String("null".into()),
            Self::Undefined => InternalPropertyKey::String("undefined".into()),
            Self::Number(n) => {
                if !n.is_nan()
                    && !n.is_infinite()
                    && n.fract() == 0.0
                    && n.is_sign_positive()
                    && n as usize <= MAX_INDEX
                {
                    InternalPropertyKey::Index(n as usize)
                } else {
                    InternalPropertyKey::String(fmt_num(n))
                }
            }
            Self::Boolean(b) => InternalPropertyKey::String(b.to_string().into()),
            Self::BigInt(b) => InternalPropertyKey::String(b.to_string().into()),
        })
    }
}

fn string_to_internal_property_key(s: YSString) -> InternalPropertyKey {
    if s.starts_with("+") || s.starts_with("-") {
        return InternalPropertyKey::String(s);
    }
    let Ok(i) = s.parse::<usize>() else {
        return InternalPropertyKey::String(s);
    };

    if i <= MAX_INDEX {
        InternalPropertyKey::Index(i)
    } else {
        InternalPropertyKey::String(s)
    }

    //TODO: this is a hack, we should not parse strings to usize
}

impl IntoPropertyKey for &Value {
    fn into_property_key(self, realm: &mut Realm) -> Res<PropertyKey> {
        self.copy().into_property_key(realm)
    }
    fn into_internal_property_key(self, realm: &mut Realm) -> Res<InternalPropertyKey> {
        self.copy().into_internal_property_key(realm)
    }
}

impl IntoPropertyKey for Symbol {
    fn into_property_key(self, _realm: &mut Realm) -> Res<PropertyKey> {
        Ok(PropertyKey::Symbol(self))
    }
    fn into_internal_property_key(self, _realm: &mut Realm) -> Res<InternalPropertyKey> {
        Ok(InternalPropertyKey::Symbol(self))
    }
}

impl IntoPropertyKey for &Symbol {
    fn into_property_key(self, _realm: &mut Realm) -> Res<PropertyKey> {
        Ok(PropertyKey::Symbol(self.clone()))
    }
    fn into_internal_property_key(self, _realm: &mut Realm) -> Res<InternalPropertyKey> {
        Ok(InternalPropertyKey::Symbol(self.clone()))
    }
}

impl IntoPropertyKey for usize {
    fn into_property_key(self, _realm: &mut Realm) -> Res<PropertyKey> {
        Ok(PropertyKey::String(self.to_string().into()))
    }
    fn into_internal_property_key(self, _realm: &mut Realm) -> Res<InternalPropertyKey> {
        Ok(InternalPropertyKey::Index(self))
    }
}

impl From<&'static str> for InternalPropertyKey {
    fn from(s: &'static str) -> Self {
        Self::String(s.into())
    }
}

impl From<String> for InternalPropertyKey {
    fn from(s: String) -> Self {
        string_to_internal_property_key(s.into())
    }
}

impl From<YSString> for InternalPropertyKey {
    fn from(s: YSString) -> Self {
        string_to_internal_property_key(s.into())
    }
}

impl From<usize> for InternalPropertyKey {
    fn from(i: usize) -> Self {
        Self::Index(i)
    }
}

impl From<Symbol> for InternalPropertyKey {
    fn from(s: Symbol) -> Self {
        Self::Symbol(s)
    }
}

impl From<&Symbol> for InternalPropertyKey {
    fn from(s: &Symbol) -> Self {
        Self::Symbol(s.clone())
    }
}

impl From<Symbol> for PropertyKey {
    fn from(s: Symbol) -> Self {
        Self::Symbol(s)
    }
}

impl From<&Symbol> for PropertyKey {
    fn from(s: &Symbol) -> Self {
        Self::Symbol(s.clone())
    }
}

impl From<&'static str> for PropertyKey {
    fn from(s: &'static str) -> Self {
        Self::String(s.into())
    }
}

impl From<String> for PropertyKey {
    fn from(s: String) -> Self {
        Self::String(s.into())
    }
}

impl From<YSString> for PropertyKey {
    fn from(s: YSString) -> Self {
        Self::String(s)
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
