
use crate::{Ctx, Object, Value};

impl<C: Ctx> From<&str> for Value<C> {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl<C: Ctx> From<String> for Value<C> {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl<C: Ctx> From<&String> for Value<C> {
    fn from(s: &String) -> Self {
        Self::String(s.clone())
    }
}

impl<C: Ctx> From<()> for Value<C> {
    fn from((): ()) -> Self {
        Self::Undefined
    }
}

impl<C: Ctx> From<f64> for Value<C> {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl<C: Ctx> From<bool> for Value<C> {
    fn from(b: bool) -> Self {
        Self::Boolean(b)
    }
}

impl<C: Ctx> From<u8> for Value<C> {
    fn from(n: u8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Ctx> From<u16> for Value<C> {
    fn from(n: u16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Ctx> From<u32> for Value<C> {
    fn from(n: u32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Ctx> From<u64> for Value<C> {
    fn from(n: u64) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Ctx> From<i8> for Value<C> {
    fn from(n: i8) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Ctx> From<i16> for Value<C> {
    fn from(n: i16) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Ctx> From<i32> for Value<C> {
    fn from(n: i32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<C: Ctx> From<i64> for Value<C> {
    fn from(n: i64) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Ctx> From<usize> for Value<C> {
    fn from(n: usize) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Ctx> From<isize> for Value<C> {
    fn from(n: isize) -> Self {
        Self::Number(n as f64)
    }
}

impl<C: Ctx> From<f32> for Value<C> {
    fn from(n: f32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl<O: Into<Object<C>>, C: Ctx> From<O> for Value<C> {
    fn from(o: O) -> Self {
        Self::Object(o.into())
    }
}
