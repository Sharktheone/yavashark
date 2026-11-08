use crate::Realm;
use crate::conversion::FromValueOutput;
use crate::value::{FromValue, IntoValue, Symbol, Value};

#[derive(Debug)]
pub enum OrSymbol<T = Value> {
    Value(T),
    Symbol(Symbol),
}

impl<T> OrSymbol<T> {
    pub const fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(_))
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> OrSymbol<U> {
        match self {
            Self::Value(v) => OrSymbol::Value(f(v)),
            Self::Symbol(s) => OrSymbol::Symbol(s),
        }
    }

    pub fn try_map<U, E, F: FnOnce(T) -> Result<U, E>>(self, f: F) -> Result<OrSymbol<U>, E> {
        match self {
            Self::Value(v) => Ok(OrSymbol::Value(f(v)?)),
            Self::Symbol(s) => Ok(OrSymbol::Symbol(s)),
        }
    }

    pub fn as_ref(&self) -> OrSymbol<&T> {
        match self {
            Self::Value(v) => OrSymbol::Value(v),
            Self::Symbol(s) => OrSymbol::Symbol(s.clone()),
        }
    }
}

impl<T: IntoValue> IntoValue for OrSymbol<T> {
    fn into_value(self) -> Value {
        match self {
            Self::Value(v) => v.into_value(),
            Self::Symbol(s) => Value::Symbol(s),
        }
    }
}

impl<T: Into<Value>> Into<Value> for OrSymbol<T> {
    fn into(self) -> Value {
        match self {
            Self::Value(v) => v.into(),
            Self::Symbol(s) => Value::Symbol(s),
        }
    }
}

impl<T> From<T> for OrSymbol<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: FromValueOutput<Output = T>> FromValueOutput for OrSymbol<T> {
    type Output = Self;

    fn from_value_out(value: Value, realm: &mut Realm) -> crate::Res<Self::Output> {
        if let Value::Symbol(sym) = value {
            Ok(Self::Symbol(sym))
        } else {
            let val = T::from_value_out(value, realm)?;
            Ok(Self::Value(val))
        }
    }
}

impl<T: FromValue> FromValue for OrSymbol<T> {
    fn from_value(value: Value) -> crate::Res<Self> {
        if let Value::Symbol(sym) = value {
            Ok(Self::Symbol(sym))
        } else {
            let val = T::from_value(value)?;
            Ok(Self::Value(val))
        }
    }
}
