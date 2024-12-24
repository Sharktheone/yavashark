use crate::{Error, Object, Realm, Value};

impl<C: Realm> From<&str> for Value<C> {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl<C: Realm> From<String> for Value<C> {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl<C: Realm> From<&String> for Value<C> {
    fn from(s: &String) -> Self {
        Self::String(s.clone())
    }
}

impl<C: Realm> From<()> for Value<C> {
    fn from((): ()) -> Self {
        Self::Undefined
    }
}

impl<C: Realm> From<f64> for Value<C> {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl<C: Realm> From<bool> for Value<C> {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl<C: Realm> From<u8> for Value<C> {
    fn from(n: u8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<u16> for Value<C> {
    fn from(n: u16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<u32> for Value<C> {
    fn from(n: u32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<u64> for Value<C> {
    fn from(n: u64) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<i8> for Value<C> {
    fn from(n: i8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<i16> for Value<C> {
    fn from(n: i16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<i32> for Value<C> {
    fn from(n: i32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Realm> From<i64> for Value<C> {
    fn from(n: i64) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<usize> for Value<C> {
    fn from(n: usize) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<isize> for Value<C> {
    fn from(n: isize) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Realm> From<f32> for Value<C> {
    fn from(n: f32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<O: Into<Object<C>>, C: Realm> From<O> for Value<C> {
    fn from(o: O) -> Self {
        Self::Object(o.into())
    }
}


pub trait FromValue<C: Realm>: Sized {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>>;
}


impl<C: Realm> FromValue<C> for String {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(Error::ty_error(format!("Expected a string, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for f64 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for bool {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Boolean(b) => Ok(b),
            _ => Err(Error::ty_error(format!("Expected a boolean, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for Object<C> {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Object(o) => Ok(o),
            _ => Err(Error::ty_error(format!("Expected an object, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for () {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Undefined => Ok(()),
            _ => Err(Error::ty_error(format!("Expected undefined, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for usize {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as usize),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for isize {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as isize),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for u8 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as u8),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for u16 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as u16),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for u32 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as u32),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for u64 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as u64),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for i8 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as i8),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for i16 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as i16),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for i32 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as i32),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for i64 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as i64),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for f32 {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        match value {
            Value::Number(n) => Ok(n as f32),
            _ => Err(Error::ty_error(format!("Expected a number, found {:?}", value))),
        }
    }
}


impl<C: Realm> FromValue<C> for Value<C> {
    fn from_value(value: Value<C>) -> Result<Self, Error<C>> {
        Ok(value)
    }
}