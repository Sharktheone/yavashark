use crate::ConstString;
use std::fmt::Display;
use std::hash::Hash;
use std::ptr;
use std::rc::Rc;
use yavashark_string::{ToYSString, YSString};

macro_rules! symbol {
    ($name:ident, $symbol:ident) => {
        pub const $name: &'static Self = &Self::new(stringify!($symbol));
    };
}

#[derive(Debug, Clone, Eq)]
pub enum SymbolInner {
    Static(&'static str),
    Str(Rc<str>),
}

impl PartialEq for SymbolInner {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(s1), Self::Static(s2)) => ptr::eq(*s1, *s2),
            (Self::Str(s1), Self::Str(s2)) => Rc::ptr_eq(s1, s2),
            _ => false,
        }
    }
}

impl Hash for SymbolInner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Static(s) => state.write_usize(s.as_ptr() as usize),
            Self::Str(s) => state.write_usize(s.as_ptr() as usize),
        }
    }
}

impl AsRef<str> for SymbolInner {
    fn as_ref(&self) -> &str {
        match self {
            Self::Static(s) => s,
            Self::Str(s) => s.as_ref(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Symbol {
    inner: SymbolInner,
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol({})", self.inner.as_ref())
    }
}

impl From<&'static str> for Symbol {
    fn from(s: &'static str) -> Self {
        Self {
            inner: SymbolInner::Static(s),
        }
    }
}

impl From<&ConstString> for Symbol {
    fn from(s: &ConstString) -> Self {
        match s {
            ConstString::String(s) => Self::new(s),
            ConstString::Owned(s) => Self::new_str(s),
        }
    }
}

impl ToYSString for Symbol {
    fn to_ys_string(&self) -> YSString {
        format!("{self}").into()
    }
}

impl Symbol {
    #[must_use]
    pub const fn new(s: &'static str) -> Self {
        Self {
            inner: SymbolInner::Static(s),
        }
    }

    #[must_use]
    pub fn new_str(s: &str) -> Self {
        Self {
            inner: SymbolInner::Str(Rc::from(s)),
        }
    }

    pub fn as_ref<'a>(&'a self) -> &'a str {
        AsRef::as_ref(self)
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
