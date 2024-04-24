use std::cmp::Ordering;
use std::ops::{Add, Div, Mul, Rem, Shl, Sub};

use super::Value;

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
            }
            (Value::Number(a), Value::Boolean(b)) => Value::Number(a - b.num()),
            (Value::Number(_), Value::Object(_)) => Value::Number(f64::NAN),

            (Value::String(a), Value::Null) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::String(a), Value::Number(b)) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a - b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::String(a), Value::Boolean(b)) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a - b.num())
                } else {
                    Value::Number(f64::NAN)
                }
            }

            (Value::String(a), Value::String(b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    Value::Number(a - b)
                } else {
                    Value::Number(f64::NAN)
                }
            }

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

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Null, _) | (_, Value::Null) => Value::Number(0.0),
            (Value::Undefined, _) | (_, Value::Undefined) => Value::Number(f64::NAN),
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            (Value::Number(a), Value::String(b))
            | (Value::String(b), Value::Number(a)) => {
                if let Ok(b) = b.parse::<f64>() {
                    Value::Number(a * b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::Number(a), Value::Boolean(b))
            | (Value::Boolean(b), Value::Number(a)) => Value::Number(a * b.num()),
            (Value::String(a), Value::String(b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    Value::Number(a * b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::String(a), Value::Boolean(b)) | (Value::Boolean(b), Value::String(a)) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a * b.num())
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::Boolean(a), Value::Boolean(b)) => Value::Number(a.num() * b.num()),
            (_, Value::Object(_)) | (Value::Object(_), _) => Value::Number(f64::NAN),
        }
    }
}

impl Div for Value {
    type Output = Self;


    //TODO: handle div by zero => return Infinity
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Null, Value::Null)
            | (Value::Null, Value::Undefined) => Value::Number(f64::NAN),
            (Value::Null, Value::Number(_)) => Value::Number(0.0),
            (Value::Null, Value::String(b)) => {
                if let Ok(b) = b.parse::<f64>() {
                    Value::Number(0.0 / b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::Null, Value::Boolean(b)) => Value::Number(0.0 / b.num()),
            (Value::Undefined, _) | (_, Value::Undefined) => Value::Number(f64::NAN),
            (Value::Number(a), Value::Null) => if a == 0.0 {
                Value::Number(f64::NAN)
            } else {
                Value::Number(f64::INFINITY)
            },
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            (Value::Number(a), Value::String(b))
            | (Value::String(b), Value::Number(a)) => {
                if let Ok(b) = b.parse::<f64>() {
                    Value::Number(a / b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::Number(a), Value::Boolean(b))
            | (Value::Boolean(b), Value::Number(a)) => Value::Number(a / b.num()),
            (Value::String(a), Value::Null) => if a == "0" {
                Value::Number(f64::NAN)
            } else {
                Value::Number(f64::INFINITY)
            },
            (Value::String(a), Value::String(b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    Value::Number(a / b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::String(a), Value::Boolean(b))
            | (Value::Boolean(b), Value::String(a)) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a / b.num())
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::Boolean(true), Value::Null) => Value::Number(f64::INFINITY),
            (Value::Boolean(false), Value::Null) => Value::Number(f64::NAN),
            (Value::Boolean(a), Value::Boolean(b)) => Value::Number(a.num() / b.num()),
            (_, Value::Object(_)) | (Value::Object(_), _) => Value::Number(f64::NAN),
        }
    }
}

impl Rem for Value {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (_, Value::Null) => Value::Number(f64::NAN),
            (Value::Null, _) => Value::Number(0.0),
            (_, Value::Undefined)
            | (Value::Undefined, _) => Value::Number(f64::NAN),
            (Value::Number(a), Value::Number(b)) => Value::Number(a % b),
            (Value::Number(a), Value::String(b)) => {
                if let Ok(b) = b.parse::<f64>() {
                    Value::Number(a % b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::Number(a), Value::Boolean(b)) => Value::Number(a % b.num()),
            (Value::String(a), Value::Number(b)) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a % b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::String(a), Value::String(b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    Value::Number(a % b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::String(a), Value::Boolean(b)) => {
                if let Ok(a) = a.parse::<f64>() {
                    Value::Number(a % b.num())
                } else {
                    Value::Number(f64::NAN)
                }
            }

            (Value::Boolean(a), Value::Number(b)) => Value::Number(a.num() % b),
            (Value::Boolean(a), Value::String(b)) => {
                if let Ok(b) = b.parse::<f64>() {
                    Value::Number(a.num() % b)
                } else {
                    Value::Number(f64::NAN)
                }
            }
            (Value::Boolean(a), Value::Boolean(b)) => Value::Number(a.num() % b.num()),
            (_, Value::Object(_)) | (Value::Object(_), _) => Value::Number(f64::NAN),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Null, Value::Null) => Some(Ordering::Equal),
            (Value::Null, Value::Number(b)) => 0.0.partial_cmp(b),
            (Value::Null, Value::String(b)) => {
                let b = b.parse::<f64>().ok()?;
                0.0.partial_cmp(&b)
            }
            (Value::Null, Value::Boolean(b)) => 0.0.partial_cmp(&b.num()),

            (Value::Undefined, _) => None,
            (_, Value::Undefined) => None,

            (Value::Number(a), Value::Null) => a.partial_cmp(&0.0),
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Number(a), Value::String(b)) => {
                let b = b.parse::<f64>().ok()?;
                a.partial_cmp(&b)
            }
            (Value::Number(a), Value::Boolean(b)) => a.partial_cmp(&b.num()),

            (Value::String(a), Value::Null) => {
                if a.is_empty() {
                    return Some(Ordering::Equal);
                }
                let a = a.parse::<f64>().ok()?;
                a.partial_cmp(&0.0)
            }
            (Value::String(a), Value::Number(b)) => a.parse::<f64>().ok()?.partial_cmp(b),
            (Value::String(a), Value::String(b)) => {
                if a == b {
                    return Some(Ordering::Equal);
                }

                let a = a.parse::<f64>().ok()?;
                let b = b.parse::<f64>().ok()?;
                a.partial_cmp(&b)
            }
            (Value::String(a), Value::Boolean(b)) => {
                let a = a.parse::<f64>().ok()?;
                a.partial_cmp(&b.num())
            }
            (Value::String(a), Value::Object(_)) => if a == "[object Object]" { Some(Ordering::Equal) } else { None },

            (Value::Boolean(a), Value::Null) => a.num().partial_cmp(&0.0),
            (Value::Boolean(a), Value::Number(b)) => a.num().partial_cmp(b),
            (Value::Boolean(a), Value::String(b)) => a.num().partial_cmp(&b.parse::<f64>().ok()?),
            (Value::Boolean(a), Value::Boolean(b)) => a.num().partial_cmp(&b.num()),

            (Value::Object(_), _) => None,
            (_, Value::Object(_)) => None,
        }
    }
}

impl Shl for Value {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Null, _) => Value::Number(0.0),
            (Value::Undefined, _) => Value::Number(0.0),

            (Value::Number(a), Value::Null) => Value::Number(a as i64 as f64),
            (Value::Number(a), Value::Undefined) => Value::Number(a as i64 as f64),
            (Value::Number(a), Value::Number(b)) => Value::Number(((a as i64) << b as i64) as f64),
            (Value::Number(a), Value::String(b)) => {
                let Ok(b) = b.parse::<f64>() else {
                    return Value::Number(0.0);
                };

                Value::Number(((a as i64) << b as i64) as f64)
            }
            (Value::Number(a), Value::Boolean(b)) => Value::Number(((a as i64) << b as i64) as f64),
            (Value::Number(a), Value::Object(_)) => Value::Number(a as i64 as f64),

            (Value::String(a), Value::Null) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Value::Number(0.0);
                };

                Value::Number(a as i64 as f64)
            }

            (Value::String(a), Value::Undefined) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Value::Number(0.0);
                };

                Value::Number(a as i64 as f64)
            }

            (Value::String(a), Value::Number(b)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Value::Number(0.0);
                };

                Value::Number(((a as i64) << b as i64) as f64)
            }

            (Value::String(a), Value::String(b)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Value::Number(0.0);
                };

                let Ok(b) = b.parse::<f64>() else {
                    return Value::Number(0.0);
                };

                Value::Number(((a as i64) << b as i64) as f64)
            }

            (Value::String(a), Value::Boolean(b)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Value::Number(0.0);
                };

                Value::Number(((a as i64) << b as i64) as f64)
            }

            (Value::String(a), Value::Object(_)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Value::Number(0.0);
                };

                Value::Number(a as i64 as f64)
            }
            
            (Value::Boolean(a), Value::Null) => Value::Number(a.num()),
            (Value::Boolean(a), Value::Undefined) => Value::Number(a.num()),
            (Value::Boolean(a), Value::Number(b)) => Value::Number(((a as i64) << b as i64) as f64),
            (Value::Boolean(a), Value::String(b)) => {
                let Ok(b) = b.parse::<f64>() else {
                    return Value::Number(0.0);
                };

                Value::Number(((a as i64) << b as i64) as f64)
            },
            
            (Value::Boolean(a), Value::Boolean(b)) => Value::Number(((a as i64) << b as i64) as f64),
            (Value::Boolean(a), Value::Object(_)) => Value::Number(a.num()),
            (Value::Object(_), _) => Value::Number(0.0),
            
        }
    }
}