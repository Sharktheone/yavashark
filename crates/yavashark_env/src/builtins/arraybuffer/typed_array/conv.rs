use half::f16;
use num_bigint::BigInt;
use crate::Value;

pub(crate) trait ArrayNumberToValue {
    fn to_value(&self) -> Value;
}


impl ArrayNumberToValue for u8 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

impl ArrayNumberToValue for u16 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

impl ArrayNumberToValue for u32 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

impl ArrayNumberToValue for u64 {
    fn to_value(&self) -> Value {
        Value::from(BigInt::from(*self))
    }
}

impl ArrayNumberToValue for i8 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

impl ArrayNumberToValue for i16 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

impl ArrayNumberToValue for i32 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

impl ArrayNumberToValue for i64 {
    fn to_value(&self) -> Value {
        Value::from(BigInt::from(*self))
    }
}

impl ArrayNumberToValue for f16 {
    fn to_value(&self) -> Value {
        Value::from(self.to_f64())
    }
}

impl ArrayNumberToValue for f32 {
    fn to_value(&self) -> Value {
        Value::from(*self as f64)
    }
}

impl ArrayNumberToValue for f64 {
    fn to_value(&self) -> Value {
        Value::from(*self)
    }
}

pub(crate) fn to_value<T: ArrayNumberToValue>(value: T) -> Value {
    value.to_value()
}
