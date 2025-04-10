use crate::ConstString;
use std::fmt::Display;
use rand::random;

macro_rules! symbol {
    ($name:ident, $symbol:ident, $id:literal) => {
        pub const $name: Self = Self::new(stringify!($symbol), $id);
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Symbol {
    inner: ConstString,
    id: u32,
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol({})", self.inner)
    }
}

impl From<&'static str> for Symbol {
    fn from(s: &'static str) -> Self {
        Self {
            inner: ConstString::String(s),
            id: random(),
        }
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Self {
            inner: ConstString::Owned(s),
            id: random(),
        }
    }
}

impl From<ConstString> for Symbol {
    fn from(s: ConstString) -> Self {
        Self { inner: s,
            id: random(),
        }
    }
}

impl Symbol {
    #[must_use]
    pub const fn new(s: &'static str, id: u32) -> Self {
        Self {
            inner: ConstString::String(s),
            id,
        }
    }
}

impl Symbol {
    symbol!(ASYNC_ITERATOR, asyncIterator, 0);
    symbol!(HAS_INSTANCE, hasInstance, 1);
    symbol!(IS_CONCAT_SPREADABLE, isConcatSpreadable, 2);
    symbol!(ITERATOR, iterator, 3);
    symbol!(MATCH, match, 4);
    symbol!(MATCH_ALL, matchAll, 5);
    symbol!(REPLACE, replace, 6);
    symbol!(SEARCH, search, 7);
    symbol!(SPECIES, species, 8);
    symbol!(SPLIT, split, 9);
    symbol!(TO_PRIMITIVE, toPrimitive, 10);
    symbol!(TO_STRING_TAG, toStringTag, 11);
    symbol!(UNSCOPABLES, unscopables, 12);
}
