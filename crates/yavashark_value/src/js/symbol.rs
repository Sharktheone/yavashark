use crate::ConstString;
use std::fmt::Display;

macro_rules! symbol {
    ($name:ident, $symbol:ident) => {
        pub const $name: Self = Self::new(stringify!($symbol));
    };
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Symbol {
    inner: ConstString,
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
        }
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Self {
            inner: ConstString::Owned(s),
        }
    }
}

impl From<ConstString> for Symbol {
    fn from(s: ConstString) -> Self {
        Self { inner: s }
    }
}

impl Symbol {
    pub const fn new(s: &'static str) -> Self {
        Self {
            inner: ConstString::String(s),
        }
    }
}

impl Symbol {
    symbol!(ASYNC_ITERATOR, asyncIterator);
    symbol!(HAS_INSTANCE, hasInstance);
    symbol!(IS_CONCAT_SPREADABLE, isConcatSpreadable);
    symbol!(ITERATOR, iterator);
    symbol!(MATCH, match);
    symbol!(MATCH_ALL, matchAll);
    symbol!(REPLACE, replace);
    symbol!(SEARCH, search);
    symbol!(SPECIES, species);
    symbol!(SPLIT, split);
    symbol!(TO_PRIMITIVE, toPrimitive);
    symbol!(TO_STRING_TAG, toStringTag);
    symbol!(UNSCOPABLES, unscopables);
}
