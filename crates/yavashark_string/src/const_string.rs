use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ConstString {
    String(&'static str),
    Owned(String),
}

impl AsRef<str> for ConstString {
    fn as_ref(&self) -> &str {
        match self {
            Self::String(s) => s,
            Self::Owned(s) => s,
        }
    }
}

impl Display for ConstString {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}")?,
            Self::Owned(s) => write!(f, "{s}")?,
        }

        Ok(())
    }
}
