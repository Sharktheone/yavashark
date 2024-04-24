use std::ops::{Add, Sub};

use crate::Value;

trait ToNumber {
    fn num(&self) -> f64;
}

impl ToNumber for bool {
    fn num(&self) -> f64 {
        if *self {
            1.0
        } else {
            0.0
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Null, Value::Null) => Value::Number(0.0),
            (Value::Null, Value::Undefined) => Value::Number(f64::NAN),
            (Value::Null, Value::Number(b)) => Value::Number(b),
            (Value::Null, Value::String(b)) => Value::String("null".to_string() + &b),
            (Value::Null, Value::Boolean(b)) => Value::Number(b.num()),
            (Value::Null, Value::Object(_)) => Value::String("null[object Object]".to_owned()),

            (Value::Undefined, Value::Null) => Value::Number(f64::NAN),
            (Value::Undefined, Value::Undefined) => Value::Number(f64::NAN),
            (Value::Undefined, Value::Number(b)) => Value::Number(b),
            (Value::Undefined, Value::String(b)) => Value::String("undefined".to_string() + &b),
            (Value::Undefined, Value::Boolean(_)) => Value::Number(f64::NAN),
            (Value::Undefined, Value::Object(_)) => Value::String("undefined[object Object]".to_owned()),

            (Value::Number(a), Value::Null) => Value::Number(a),
            (Value::Number(_), Value::Undefined) => Value::Number(f64::NAN),
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            (Value::Number(a), Value::String(b)) => Value::String(a.to_string() + &b),
            (Value::Number(a), Value::Boolean(b)) => Value::Number(a + b.num()),
            (Value::Number(a), Value::Object(_)) => Value::String(a.to_string() + "[object Object]"),

            (Value::String(a), Value::Null) => Value::String(a + "null"),
            (Value::String(a), Value::Undefined) => Value::String(a + "undefined"),
            (Value::String(a), Value::Number(b)) => Value::String(a + &b.to_string()),
            (Value::String(a), Value::String(b)) => Value::String(a + &b),
            (Value::String(a), Value::Boolean(b)) => Value::String(a + &b.to_string()),
            (Value::String(a), Value::Object(_)) => Value::String(a + "[object Object]"),

            (Value::Boolean(a), Value::Null) => Value::Number(a.num()),
            (Value::Boolean(_), Value::Undefined) => Value::Number(f64::NAN),
            (Value::Boolean(a), Value::Number(b)) => Value::Number(a.num() + b),
            (Value::Boolean(a), Value::String(b)) => Value::String(a.to_string() + &b),
            (Value::Boolean(a), Value::Boolean(b)) => Value::Number(a.num() + b.num()),
            (Value::Boolean(a), Value::Object(_)) => Value::String(a.to_string() + "[object Object]"),

            (Value::Object(_), Value::Null) => Value::String("[object Object]null".to_owned()),
            (Value::Object(_), Value::Undefined) => Value::String("[object Object]undefined".to_owned()),
            (Value::Object(_), Value::Number(b)) => Value::String("[object Object]".to_owned() + &b.to_string()),
            (Value::Object(_), Value::String(b)) => Value::String("[object Object]".to_owned() + &b),
            (Value::Object(_), Value::Boolean(b)) => Value::String("[object Object]".to_owned() + &b.to_string()),
            (Value::Object(_), Value::Object(_)) => Value::String("[object Object][object Object]".to_owned()),
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Null, Value::Null) => Value::Number(0.0),
            (Value::Null, Value::Undefined) => Value::Number(f64::NAN),
            (Value::Null, Value::Number(b)) => Value::Number(-b),
            (Value::Null, Value::String(_)) => Value::Number(f64::NAN),
            (Value::Null, Value::Boolean(b)) => Value::Number(-b.num()),
            (Value::Null, Value::Object(_)) => Value::Number(f64::NAN),

            (Value::Undefined, _) => Value::Number(f64::NAN),

            (Value::Number(a), Value::Null) => Value::Number(a),
            (Value::Number(_), Value::Undefined) => Value::Number(f64::NAN),
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            (Value::Number(a), Value::String(b)) => {
                if let Ok(b) = b.parse::<f64>() {
                    Value::Number(a - b)
                } else {
                    Value::Number(f64::NAN)
                }
            },
            (Value::Number(a), Value::Boolean(b)) => Value::Number(a - b.num()),
            (Value::Number(a), Value::Object(_)) => Value::Number(f64::NAN),

            (Value::String(a), Value::Null) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a)
                } else {
                    Value::Number(f64::NAN)
                }
            },
            (Value::String(a), Value::Number(b)) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a - b)
                } else {
                    Value::Number(f64::NAN)
                }
            },
            (Value::String(a), Value::Boolean(b)) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a - b.num())
                } else {
                    Value::Number(f64::NAN)
                }
            },
            (Value::String(_), _) => Value::Number(f64::NAN),

            (Value::Boolean(a), Value::Null) => Value::Number(a.num()),
            (Value::Boolean(_), Value::Undefined) => Value::Number(f64::NAN),
            (Value::Boolean(a), Value::Number(b)) => Value::Number(a.num() - b),
            (Value::Boolean(_), Value::String(_)) => Value::Number(f64::NAN),
            (Value::Boolean(a), Value::Boolean(b)) => Value::Number(a.num() - b.num()),
            (Value::Boolean(_), Value::Object(_)) => Value::Number(f64::NAN),

            (Value::Object(_), _) => Value::Number(f64::NAN),
        }
    }
}