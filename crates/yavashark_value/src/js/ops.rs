#![allow(clippy::match_same_arms)]

use std::cmp::Ordering;
use std::ops::{Add, AddAssign, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub, SubAssign};

use crate::Ctx;

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

impl<C: Ctx> Value<C> {
    #[must_use]
    pub fn to_number_or_null(&self) -> f64 {
        match self {
            Self::Number(n) => *n,
            Self::Boolean(b) => b.num(),
            Self::String(s) => s.parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    #[must_use]
    pub fn to_number(&self) -> f64 {
        match self {
            Self::Number(n) => *n,
            Self::Boolean(b) => b.num(),
            Self::String(s) => {
                if s.is_empty() {
                    0.0
                } else {
                    s.parse().unwrap_or(f64::NAN)
                }
            }
            Self::Symbol(_) => todo!("return a Result here.... to throw an TypeError"),
            _ => f64::NAN,
        }
    }

    #[must_use]
    pub fn to_int_or_null(&self) -> i64 {
        match self {
            Self::Number(n) => *n as i64,
            Self::Boolean(b) => i64::from(*b),
            Self::String(s) => s.parse().unwrap_or(0),
            Self::Symbol(_) => todo!("return a Result here.... to throw an TypeError"),
            _ => 0,
        }
    }
}

impl<C: Ctx> Add for Value<C> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Null, Self::Null) => Self::Number(0.0),
            (Self::Null, Self::Undefined) => Self::Number(f64::NAN),
            (Self::Null, Self::Number(b)) => Self::Number(b),
            (Self::Null, Self::String(b)) => Self::String("null".to_string() + &b),
            (Self::Null, Self::Boolean(b)) => Self::Number(b.num()),
            (Self::Null, Self::Object(_)) => Self::String("null[object Object]".to_owned()),

            (Self::Undefined, Self::Null) => Self::Number(f64::NAN),
            (Self::Undefined, Self::Undefined) => Self::Number(f64::NAN),
            (Self::Undefined, Self::Number(_)) => Self::Number(f64::NAN),
            (Self::Undefined, Self::String(b)) => Self::String("undefined".to_string() + &b),
            (Self::Undefined, Self::Boolean(_)) => Self::Number(f64::NAN),
            (Self::Undefined, Self::Object(_)) => {
                Self::String("undefined[object Object]".to_owned())
            }

            (Self::Number(a), Self::Null) => Self::Number(a),
            (Self::Number(_), Self::Undefined) => Self::Number(f64::NAN),
            (Self::Number(a), Self::Number(b)) => Self::Number(a + b),
            (Self::Number(a), Self::String(b)) => Self::String(a.to_string() + &b),
            (Self::Number(a), Self::Boolean(b)) => Self::Number(a + b.num()),
            (Self::Number(a), Self::Object(_)) => Self::String(a.to_string() + "[object Object]"),

            (Self::String(a), Self::Null) => Self::String(a + "null"),
            (Self::String(a), Self::Undefined) => Self::String(a + "undefined"),
            (Self::String(a), Self::Number(b)) => Self::String(a + &b.to_string()),
            (Self::String(a), Self::String(b)) => Self::String(a + &b),
            (Self::String(a), Self::Boolean(b)) => Self::String(a + &b.to_string()),
            (Self::String(a), Self::Object(_)) => Self::String(a + "[object Object]"),

            (Self::Boolean(a), Self::Null) => Self::Number(a.num()),
            (Self::Boolean(_), Self::Undefined) => Self::Number(f64::NAN),
            (Self::Boolean(a), Self::Number(b)) => Self::Number(a.num() + b),
            (Self::Boolean(a), Self::String(b)) => Self::String(a.to_string() + &b),
            (Self::Boolean(a), Self::Boolean(b)) => Self::Number(a.num() + b.num()),
            (Self::Boolean(a), Self::Object(o)) => Self::String(a.to_string() + &o.to_string()),

            (Self::Symbol(_), _) | (_, Self::Symbol(_)) => {
                todo!("return a Result here.... to throw an TypeError")
            }
            (a, b) => Self::String(a.to_string() + &b.to_string()),
        }
    }
}

impl<C: Ctx> Sub for Value<C> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Null, Self::Null) => Self::Number(0.0),
            (Self::Null, Self::Number(b)) => Self::Number(-b),
            (Self::Null, Self::String(b)) => b
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |b| Self::Number(-b)),
            (Self::Null, Self::Boolean(b)) => Self::Number(-b.num()),

            (Self::Number(a), Self::Null) => Self::Number(a),
            (Self::Number(a), Self::Number(b)) => Self::Number(a - b),
            (Self::Number(a), Self::String(b)) => b
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |b| Self::Number(a - b)),
            (Self::Number(a), Self::Boolean(b)) => Self::Number(a - b.num()),

            (Self::String(a), Self::Null) => a
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |a| Self::Number(a)),
            (Self::String(a), Self::Number(b)) => a
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |a| Self::Number(a - b)),
            (Self::String(a), Self::Boolean(b)) => a
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |a| Self::Number(a - b.num())),

            (Self::String(a), Self::String(b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    Self::Number(a - b)
                } else {
                    Self::Number(f64::NAN)
                }
            }

            (Self::String(_), _) => Self::Number(f64::NAN),

            (Self::Boolean(a), Self::Null) => Self::Number(a.num()),
            (Self::Boolean(a), Self::Number(b)) => Self::Number(a.num() - b),
            (Self::Boolean(a), Self::String(b)) => b
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |b| Self::Number(a.num() - b)),
            (Self::Boolean(a), Self::Boolean(b)) => Self::Number(a.num() - b.num()),
            (Self::Boolean(_), Self::Object(_)) => Self::Number(f64::NAN),

            (Self::Object(_), _) | (_, Self::Object(_)) => Self::Number(f64::NAN),
            (Self::Undefined, _) | (_, Self::Undefined) => Self::Number(f64::NAN),
            (Self::Symbol(_), _) | (_, Self::Symbol(_)) => {
                todo!("return a Result here.... to throw an TypeError")
            }
        }
    }
}

impl<C: Ctx> Mul for Value<C> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Null, Self::String(b)) => {
                if b.parse::<f64>().is_ok() {
                    Self::Number(0.0)
                } else {
                    Self::Number(f64::NAN)
                }
            }
            (Self::Undefined, _) | (_, Self::Undefined) => Self::Number(f64::NAN),
            (_, Self::Object(_)) | (Self::Object(_), _) => Self::Number(f64::NAN),
            (Self::Null, _) | (_, Self::Null) => Self::Number(0.0),
            (Self::Number(a), Self::Number(b)) => Self::Number(a * b),
            (Self::Number(a), Self::String(b)) | (Self::String(b), Self::Number(a)) => b
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |b| Self::Number(a * b)),
            (Self::Number(a), Self::Boolean(b)) | (Self::Boolean(b), Self::Number(a)) => {
                Self::Number(a * b.num())
            }
            (Self::String(a), Self::String(b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    Self::Number(a * b)
                } else {
                    Self::Number(f64::NAN)
                }
            }
            (Self::String(a), Self::Boolean(b)) | (Self::Boolean(b), Self::String(a)) => a
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |a| Self::Number(a * b.num())),
            (Self::Boolean(a), Self::Boolean(b)) => Self::Number(a.num() * b.num()),
            (Self::Symbol(_), _) | (_, Self::Symbol(_)) => {
                todo!("return a Result here.... to throw an TypeError")
            }
        }
    }
}

impl<C: Ctx> Div for Value<C> {
    type Output = Self;

    //TODO: handle div by zero => return Infinity
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Null, Self::Null | Self::Undefined) => Self::Number(f64::NAN),
            (Self::Null, Self::Number(_)) => Self::Number(0.0),
            (Self::Null, Self::String(b)) => b
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |b| Self::Number(0.0 / b)),
            (Self::Null, Self::Boolean(b)) => Self::Number(0.0 / b.num()),
            (Self::Undefined, _) | (_, Self::Undefined) => Self::Number(f64::NAN),
            (Self::Number(a), Self::Null) => {
                if a == 0.0 {
                    Self::Number(f64::NAN)
                } else {
                    Self::Number(f64::INFINITY)
                }
            }
            (Self::Number(a), Self::Number(b)) => Self::Number(a / b),
            (Self::Number(a), Self::String(b)) => b
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |b| Self::Number(a / b)),
            (Self::Number(a), Self::Boolean(b)) => Self::Number(a / b.num()),
            (Self::String(a), Self::Null) => {
                if a == "0" || a == "0.0" || a.parse::<f64>().is_err() {
                    Self::Number(f64::NAN)
                } else {
                    Self::Number(f64::INFINITY)
                }
            }

            (Self::String(a), Self::Number(b)) => a
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |a| Self::Number(a / b)),

            (Self::String(a), Self::String(b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    Self::Number(a / b)
                } else {
                    Self::Number(f64::NAN)
                }
            }
            (Self::String(a), Self::Boolean(b)) | (Self::Boolean(b), Self::String(a)) => a
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |a| Self::Number(a / b.num())),
            (Self::Boolean(true), Self::Null) => Self::Number(f64::INFINITY),
            (Self::Boolean(false), Self::Null) => Self::Number(f64::NAN),
            (Self::Boolean(a), Self::Number(b)) => Self::Number(a.num() / b),
            (Self::Boolean(a), Self::Boolean(b)) => Self::Number(a.num() / b.num()),
            (_, Self::Object(_)) | (Self::Object(_), _) => Self::Number(f64::NAN),
            (Self::Symbol(_), _) | (_, Self::Symbol(_)) => {
                todo!("return a Result here.... to throw an TypeError")
            }
        }
    }
}

impl<C: Ctx> Rem for Value<C> {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (_, Self::Null) => Self::Number(f64::NAN),
            (Self::Null, Self::Object(_)) => Self::Number(f64::NAN),
            (Self::Null, Self::Undefined) => Self::Number(f64::NAN),
            (Self::Null, Self::String(b)) => b
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |b| Self::Number(0.0 % b)),
            (Self::Null, _) => Self::Number(0.0),
            (_, Self::Undefined) | (Self::Undefined, _) => Self::Number(f64::NAN),
            (Self::Number(a), Self::Number(b)) => Self::Number(a % b),
            (Self::Number(a), Self::String(b)) => b
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |b| Self::Number(a % b)),
            (Self::Number(a), Self::Boolean(b)) => Self::Number(a % b.num()),
            (Self::String(a), Self::Number(b)) => a
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |a| Self::Number(a % b)),
            (Self::String(a), Self::String(b)) => {
                if let (Ok(a), Ok(b)) = (a.parse::<f64>(), b.parse::<f64>()) {
                    Self::Number(a % b)
                } else {
                    Self::Number(f64::NAN)
                }
            }
            (Self::String(a), Self::Boolean(b)) => a
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |a| Self::Number(a % b.num())),

            (Self::Boolean(a), Self::Number(b)) => Self::Number(a.num() % b),
            (Self::Boolean(a), Self::String(b)) => b
                .parse::<f64>()
                .map_or_else(|_| Self::Number(f64::NAN), |b| Self::Number(a.num() % b)),
            (Self::Boolean(a), Self::Boolean(b)) => Self::Number(a.num() % b.num()),
            (_, Self::Object(_)) | (Self::Object(_), _) => Self::Number(f64::NAN),
            (Self::Symbol(_), _) | (_, Self::Symbol(_)) => {
                todo!("return a Result here.... to throw an TypeError")
            }
        }
    }
}

impl<C: Ctx> PartialOrd for Value<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Null, Self::Null) => Some(Ordering::Equal),
            (Self::Null, Self::Number(b)) => 0.0.partial_cmp(b),
            (Self::Null, Self::String(b)) => {
                let b = b.parse::<f64>().ok()?;
                0.0.partial_cmp(&b)
            }
            (Self::Null, Self::Boolean(b)) => 0.0.partial_cmp(&b.num()),

            (Self::Undefined, _) => None,
            (_, Self::Undefined) => None,

            (Self::Number(a), Self::Null) => a.partial_cmp(&0.0),
            (Self::Number(a), Self::Number(b)) => a.partial_cmp(b),
            (Self::Number(a), Self::String(b)) => {
                let b = b.parse::<f64>().ok()?;
                a.partial_cmp(&b)
            }
            (Self::Number(a), Self::Boolean(b)) => a.partial_cmp(&b.num()),

            (Self::String(a), Self::Null) => {
                if a.is_empty() {
                    return Some(Ordering::Equal);
                }
                let a = a.parse::<f64>().ok()?;
                a.partial_cmp(&0.0)
            }
            (Self::String(a), Self::Number(b)) => a.parse::<f64>().ok()?.partial_cmp(b),
            (Self::String(a), Self::String(b)) => {
                if a == b {
                    return Some(Ordering::Equal);
                }

                let a = a.parse::<f64>().ok()?;
                let b = b.parse::<f64>().ok()?;
                a.partial_cmp(&b)
            }
            (Self::String(a), Self::Boolean(b)) => {
                let a = a.parse::<f64>().ok()?;
                a.partial_cmp(&b.num())
            }
            (Self::String(a), Self::Object(_)) => {
                if a == "[object Object]" {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }

            (Self::Boolean(a), Self::Null) => a.num().partial_cmp(&0.0),
            (Self::Boolean(a), Self::Number(b)) => a.num().partial_cmp(b),
            (Self::Boolean(a), Self::String(b)) => a.num().partial_cmp(&b.parse::<f64>().ok()?),
            (Self::Boolean(a), Self::Boolean(b)) => a.num().partial_cmp(&b.num()),

            (Self::Object(_), _) => None,
            (_, Self::Object(_)) => None,
            (Self::Symbol(_), _) | (_, Self::Symbol(_)) => {
                todo!("return a Result here.... to throw an TypeError")
            }
        }
    }
}

impl<C: Ctx> Shl for Value<C> {
    type Output = Self;

    fn shl(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Null, _) => Self::Number(0.0),
            (Self::Undefined, _) => Self::Number(0.0),

            (Self::Number(a), Self::Null) => Self::Number(a as i64 as f64),
            (Self::Number(a), Self::Undefined) => Self::Number(a as i64 as f64),
            (Self::Number(a), Self::Number(b)) => Self::Number(((a as i64) << b as i64) as f64),
            (Self::Number(a), Self::String(b)) => {
                let Ok(b) = b.parse::<f64>() else {
                    return Self::Number(a as i64 as f64);
                };

                Self::Number(((a as i64) << b as i64) as f64)
            }
            (Self::Number(a), Self::Boolean(b)) => {
                Self::Number(((a as i64) << i64::from(b)) as f64)
            }
            (Self::Number(a), Self::Object(_)) => Self::Number(a as i64 as f64),

            (Self::String(a), Self::Null) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(a as i64 as f64)
            }

            (Self::String(a), Self::Undefined) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(a as i64 as f64)
            }

            (Self::String(a), Self::Number(b)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(((a as i64) << b as i64) as f64)
            }

            (Self::String(a), Self::String(b)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                let Ok(b) = b.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(((a as i64) << b as i64) as f64)
            }

            (Self::String(a), Self::Boolean(b)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(((a as i64) << i64::from(b)) as f64)
            }

            (Self::String(a), Self::Object(_)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(a as i64 as f64)
            }

            (Self::Boolean(a), Self::Null) => Self::Number(a.num()),
            (Self::Boolean(a), Self::Undefined) => Self::Number(a.num()),
            (Self::Boolean(a), Self::Number(b)) => Self::Number((i64::from(a) << b as i64) as f64),
            (Self::Boolean(a), Self::String(b)) => {
                let Ok(b) = b.parse::<f64>() else {
                    return Self::Number(a.num());
                };

                Self::Number((i64::from(a) << b as i64) as f64)
            }

            (Self::Boolean(a), Self::Boolean(b)) => {
                Self::Number((i64::from(a) << i64::from(b)) as f64)
            }
            (Self::Boolean(a), Self::Object(_)) => Self::Number(a.num()),
            (Self::Object(_), _) => Self::Number(0.0),
            (Self::Symbol(_), _) | (_, Self::Symbol(_)) => {
                todo!("return a Result here.... to throw an TypeError")
            }
        }
    }
}

impl<C: Ctx> Shr for Value<C> {
    type Output = Self;

    fn shr(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Null, _) => Self::Number(0.0),
            (Self::Undefined, _) => Self::Number(0.0),

            (Self::Number(a), Self::Null) => Self::Number(a as i64 as f64),
            (Self::Number(a), Self::Undefined) => Self::Number(a as i64 as f64),
            (Self::Number(a), Self::Number(b)) => Self::Number(((a as i64) >> b as i64) as f64),
            (Self::Number(a), Self::String(b)) => {
                let Ok(b) = b.parse::<f64>() else {
                    return Self::Number(a as i64 as f64);
                };

                Self::Number(((a as i64) >> b as i64) as f64)
            }
            (Self::Number(a), Self::Boolean(b)) => {
                Self::Number(((a as i64) >> i64::from(b)) as f64)
            }
            (Self::Number(a), Self::Object(_)) => Self::Number(a as i64 as f64),

            (Self::String(a), Self::Null) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(a as i64 as f64)
            }

            (Self::String(a), Self::Undefined) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(a as i64 as f64)
            }

            (Self::String(a), Self::Number(b)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(((a as i64) >> b as i64) as f64)
            }

            (Self::String(a), Self::String(b)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                let Ok(b) = b.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(((a as i64) >> b as i64) as f64)
            }

            (Self::String(a), Self::Boolean(b)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(((a as i64) >> i64::from(b)) as f64)
            }

            (Self::String(a), Self::Object(_)) => {
                let Ok(a) = a.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number(a as i64 as f64)
            }

            (Self::Boolean(a), Self::Null) => Self::Number(a.num()),
            (Self::Boolean(a), Self::Undefined) => Self::Number(a.num()),
            (Self::Boolean(a), Self::Number(b)) => Self::Number((i64::from(a) >> b as i64) as f64),
            (Self::Boolean(a), Self::String(b)) => {
                let Ok(b) = b.parse::<f64>() else {
                    return Self::Number(0.0);
                };

                Self::Number((i64::from(a) >> b as i64) as f64)
            }

            (Self::Boolean(a), Self::Boolean(b)) => {
                Self::Number((i64::from(a) >> i64::from(b)) as f64)
            }
            (Self::Boolean(a), Self::Object(_)) => Self::Number(a.num()),
            (Self::Object(_), _) => Self::Number(0.0),
            (Self::Symbol(_), _) | (_, Self::Symbol(_)) => {
                todo!("return a Result here.... to throw an TypeError")
            }
        }
    }
}

impl<C: Ctx> BitOr for Value<C> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::Number((self.to_int_or_null() | rhs.to_int_or_null()) as f64)
    }
}

impl<C: Ctx> BitAnd for Value<C> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::Number((self.to_int_or_null() & rhs.to_int_or_null()) as f64)
    }
}

impl<C: Ctx> BitXor for Value<C> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::Number((self.to_int_or_null() ^ rhs.to_int_or_null()) as f64)
    }
}

impl<C: Ctx> Value<C> {
    #[must_use]
    pub fn log_or(&self, rhs: Self) -> Self {
        if self.is_truthy() {
            self.copy()
        } else {
            rhs
        }
    }

    #[must_use]
    pub fn log_and(&self, rhs: Self) -> Self {
        if self.is_truthy() {
            rhs
        } else {
            self.copy()
        }
    }

    #[must_use]
    pub fn pow(&self, rhs: &Self) -> Self {
        Self::Number(self.to_number().powf(rhs.to_number()))
    }

    #[must_use]
    pub fn normal_eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Self::Null, Self::Null) | (Self::Undefined, Self::Undefined) => true,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => a == b,
            (Self::Symbol(a), Self::Symbol(b)) => a == b,

            (Self::Number(a), Self::String(b)) | (Self::String(b), Self::Number(a)) => {
                a.to_string() == *b
            }

            (Self::Number(a), Self::Boolean(b)) | (Self::Boolean(b), Self::Number(a)) => {
                *a == b.num()
            }

            (Self::Number(a), Self::Object(b)) | (Self::Object(b), Self::Number(a)) => {
                a.to_string() == b.to_string()
            }

            (Self::String(a), Self::Object(b)) | (Self::Object(b), Self::String(a)) => {
                *a == b.to_string()
            }

            (Self::String(a), Self::Boolean(b)) | (Self::Boolean(b), Self::String(a)) => {
                *a == b.num().to_string()
            }

            (Self::Boolean(a), Self::Object(b)) | (Self::Object(b), Self::Boolean(a)) => {
                a.num().to_string() == b.to_string()
            }

            _ => false,
        }
    }
}

impl<C: Ctx> AddAssign for Value<C> {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.copy() + rhs; //TODO: don't copy the value
    }
}

impl<C: Ctx> SubAssign for Value<C> {
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.copy() - rhs; //TODO: don't copy the value
    }
}

#[cfg(test)]
mod tests {
    use crate::variable::Variable;
    use crate::Obj;

    use super::*;

    type Value = super::Value<()>;
    type Error = crate::Error<()>;

    #[derive(Debug, PartialEq)]
    struct Object;

    impl Ctx for () {}

    impl Obj<()> for Object {
        fn define_property(&mut self, _name: Value, _value: Value) {}

        fn define_variable(&mut self, _name: crate::Value<()>, _value: Variable<()>) {}

        fn resolve_property(&self, _name: &Value) -> Option<Value> {
            None
        }

        fn get_property(&self, _name: &Value) -> Option<&Value> {
            None
        }

        fn get_property_mut(&mut self, _name: &Value) -> Option<&mut Value> {
            None
        }

        fn delete_property(&mut self, name: &crate::Value<()>) -> Option<crate::Value<()>> {
            None
        }

        fn name(&self) -> String {
            "Object".to_string()
        }

        fn to_string(&self) -> String {
            format!("[object {}]", self.name())
        }

        fn properties(&self) -> Vec<(Value, Value)> {
            vec![]
        }

        fn keys(&self) -> Vec<crate::Value<()>> {
            vec![]
        }

        fn values(&self) -> Vec<crate::Value<()>> {
            vec![]
        }

        fn get_array_or_done(&self, _index: usize) -> (bool, Option<crate::Value<()>>) {
            (false, None)
        }

        fn clear_values(&mut self) {}
    }

    impl From<Object> for crate::Object<()> {
        fn from(obj: Object) -> Self {
            let boxed: Box<dyn Obj<()>> = Box::new(obj);
            Self::from_boxed(boxed)
        }
    }

    #[test]
    fn add_null_null() {
        let a = Value::Null;
        let b = Value::Null;
        assert_eq!(a + b, Value::Number(0.0));
    }

    #[test]
    fn add_null_undefined() {
        let a = Value::Null;
        let b = Value::Undefined;
        assert!((a + b).is_nan());
    }

    #[test]
    fn add_null_number() {
        let a = Value::Null;
        let b = Value::Number(1.0);
        assert_eq!(a + b, Value::Number(1.0));
    }

    #[test]
    fn add_null_string() {
        let a = Value::Null;
        let b = Value::String("hello".to_string());
        assert_eq!(a + b, Value::String("nullhello".to_string()));

        let a = Value::Null;
        let b = Value::String("1".to_string());
        assert_eq!(a + b, Value::String("null1".to_string()));
    }

    #[test]
    fn add_null_boolean() {
        let a = Value::Null;
        let b = Value::Boolean(true);
        assert_eq!(a + b, Value::Number(1.0));

        let a = Value::Null;
        let b = Value::Boolean(false);
        assert_eq!(a + b, Value::Number(0.0));
    }

    #[test]
    fn add_null_object() {
        let a = Value::Null;
        let b = Value::Object(Object.into());
        assert_eq!(a + b, Value::String("null[object Object]".to_string()));
    }

    #[test]
    fn add_undefined_null() {
        let a = Value::Undefined;
        let b = Value::Null;
        assert!((a + b).is_nan());
    }

    #[test]
    fn add_undefined_undefined() {
        let a = Value::Undefined;
        let b = Value::Undefined;
        assert!((a + b).is_nan());
    }

    #[test]
    fn add_undefined_number() {
        let a = Value::Undefined;
        let b = Value::Number(1.0);
        assert!((a + b).is_nan());
    }

    #[test]
    fn add_undefined_string() {
        let a = Value::Undefined;
        let b = Value::String("hello".to_string());
        assert_eq!(a + b, Value::String("undefinedhello".to_string()));

        let a = Value::Undefined;
        let b = Value::String("1".to_string());
        assert_eq!(a + b, Value::String("undefined1".to_string()));
    }

    #[test]
    fn add_undefined_boolean() {
        let a = Value::Undefined;
        let b = Value::Boolean(true);
        assert!((a + b).is_nan());

        let a = Value::Undefined;
        let b = Value::Boolean(false);
        assert!((a + b).is_nan());
    }

    #[test]
    fn add_undefined_object() {
        let a = Value::Undefined;
        let b = Value::Object(Object.into());
        assert_eq!(a + b, Value::String("undefined[object Object]".to_string()));
    }

    #[test]
    fn add_number_null() {
        let a = Value::Number(1.0);
        let b = Value::Null;
        assert_eq!(a + b, Value::Number(1.0));
    }

    #[test]
    fn add_number_undefined() {
        let a = Value::Number(1.0);
        let b = Value::Undefined;
        assert!((a + b).is_nan());
    }

    #[test]
    fn add_numbers_number() {
        let a = Value::Number(1.0);
        let b = Value::Number(2.0);
        assert_eq!(a + b, Value::Number(3.0));
    }

    #[test]
    fn add_numbers_string() {
        let a = Value::Number(1.0);
        let b = Value::String("2".to_string());
        assert_eq!(a + b, Value::String("12".to_string()));

        let a = Value::Number(1.0);
        let b = Value::String("a".to_string());
        assert_eq!(a + b, Value::String("1a".to_string()));
    }

    #[test]
    fn add_number_boolean() {
        let a = Value::Number(1.0);
        let b = Value::Boolean(true);
        assert_eq!(a + b, Value::Number(2.0));

        let a = Value::Number(1.0);
        let b = Value::Boolean(false);
        assert_eq!(a + b, Value::Number(1.0));
    }

    #[test]
    fn add_number_object() {
        let a = Value::Number(1.0);
        let b = Value::Object(Object.into());
        assert_eq!(a + b, Value::String("1[object Object]".to_string()));
    }

    #[test]
    fn add_string_null() {
        let a = Value::String("hello".to_string());
        let b = Value::Null;
        assert_eq!(a + b, Value::String("hellonull".to_string()));
    }

    #[test]
    fn add_string_undefined() {
        let a = Value::String("hello".to_string());
        let b = Value::Undefined;
        assert_eq!(a + b, Value::String("helloundefined".to_string()));
    }

    #[test]
    fn add_string_number() {
        let a = Value::String("1".to_string());
        let b = Value::Number(2.0);
        assert_eq!(a + b, Value::String("12".to_string()));
    }

    #[test]
    fn add_string_string() {
        let a = Value::String("1".to_string());
        let b = Value::String("2".to_string());
        assert_eq!(a + b, Value::String("12".to_string()));

        let a = Value::String("hello".to_string());
        let b = Value::String("world".to_string());
        assert_eq!(a + b, Value::String("helloworld".to_string()));
    }

    #[test]
    fn add_string_boolean() {
        let a = Value::String("1".to_string());
        let b = Value::Boolean(true);
        assert_eq!(a + b, Value::String("1true".to_string()));

        let a = Value::String("hello".to_string());
        let b = Value::Boolean(true);
        assert_eq!(a + b, Value::String("hellotrue".to_string()));
    }

    #[test]
    fn add_string_object() {
        let a = Value::String("hello".to_string());
        let b = Value::Object(Object.into());
        assert_eq!(a + b, Value::String("hello[object Object]".to_string()));
    }

    #[test]
    fn add_boolean_null() {
        let a = Value::Boolean(true);
        let b = Value::Null;
        assert_eq!(a + b, Value::Number(1.0));

        let a = Value::Boolean(false);
        let b = Value::Null;
        assert_eq!(a + b, Value::Number(0.0));
    }

    #[test]
    fn add_boolean_undefined() {
        let a = Value::Boolean(true);
        let b = Value::Undefined;
        assert!((a + b).is_nan());

        let a = Value::Boolean(false);
        let b = Value::Undefined;
        assert!((a + b).is_nan());
    }

    #[test]
    fn add_boolean_number() {
        let a = Value::Boolean(true);
        let b = Value::Number(2.0);
        assert_eq!(a + b, Value::Number(3.0));

        let a = Value::Boolean(false);
        let b = Value::Number(2.0);
        assert_eq!(a + b, Value::Number(2.0));
    }

    #[test]
    fn add_boolean_string() {
        let a = Value::Boolean(true);
        let b = Value::String("2".to_string());
        assert_eq!(a + b, Value::String("true2".to_string()));

        let a = Value::Boolean(false);
        let b = Value::String("2".to_string());
        assert_eq!(a + b, Value::String("false2".to_string()));
    }

    #[test]
    fn add_boolean_boolean() {
        let a = Value::Boolean(true);
        let b = Value::Boolean(true);
        assert_eq!(a + b, Value::Number(2.0));

        let a = Value::Boolean(false);
        let b = Value::Boolean(true);
        assert_eq!(a + b, Value::Number(1.0));

        let a = Value::Boolean(false);
        let b = Value::Boolean(false);
        assert_eq!(a + b, Value::Number(0.0));
    }

    #[test]
    fn add_boolean_object() {
        let a = Value::Boolean(true);
        let b = Value::Object(Object.into());
        assert_eq!(a + b, Value::String("true[object Object]".to_string()));

        let a = Value::Boolean(false);
        let b = Value::Object(Object.into());
        assert_eq!(a + b, Value::String("false[object Object]".to_string()));
    }

    #[test]
    fn add_object_null() {
        let a = Value::Object(Object.into());
        let b = Value::Null;
        assert_eq!(a + b, Value::String("[object Object]null".to_string()));
    }

    #[test]
    fn add_object_undefined() {
        let a = Value::Object(Object.into());
        let b = Value::Undefined;
        assert_eq!(a + b, Value::String("[object Object]undefined".to_string()));
    }

    #[test]
    fn add_object_number() {
        let a = Value::Object(Object.into());
        let b = Value::Number(1.0);
        assert_eq!(a + b, Value::String("[object Object]1".to_string()));
    }

    #[test]
    fn add_object_string() {
        let a = Value::Object(Object.into());
        let b = Value::String("hello".to_string());
        assert_eq!(a + b, Value::String("[object Object]hello".to_string()));
    }

    #[test]
    fn add_object_boolean() {
        let a = Value::Object(Object.into());
        let b = Value::Boolean(true);
        assert_eq!(a + b, Value::String("[object Object]true".to_string()));
    }

    #[test]
    fn add_object_object() {
        let a = Value::Object(Object.into());
        let b = Value::Object(Object.into());
        assert_eq!(
            a + b,
            Value::String("[object Object][object Object]".to_string())
        );
    }

    #[test]
    fn sub_null_null() {
        let a = Value::Null;
        let b = Value::Null;
        assert_eq!(a - b, Value::Number(0.0));
    }

    #[test]
    fn sub_null_undefined() {
        let a = Value::Null;
        let b = Value::Undefined;
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_null_number() {
        let a = Value::Null;
        let b = Value::Number(1.0);
        assert_eq!(a - b, Value::Number(-1.0));
    }

    #[test]
    fn sub_null_string() {
        let a = Value::Null;
        let b = Value::String("1".to_string());
        assert_eq!(a - b, Value::Number(-1.0));

        let a = Value::Null;
        let b = Value::String("a".to_string());
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_null_boolean() {
        let a = Value::Null;
        let b = Value::Boolean(true);
        assert_eq!(a - b, Value::Number(-1.0));
    }

    #[test]
    fn sub_null_object() {
        let a = Value::Null;
        let b = Value::Object(Object.into());
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_undefined_any() {
        let a = Value::Undefined;
        let b = Value::Null;
        assert!((a - b).is_nan());

        let a = Value::Undefined;
        let b = Value::Undefined;
        assert!((a - b).is_nan());

        let a = Value::Undefined;
        let b = Value::Number(1.0);
        assert!((a - b).is_nan());

        let a = Value::Undefined;
        let b = Value::String("a".to_string());
        assert!((a - b).is_nan());

        let a = Value::Undefined;
        let b = Value::String("1".to_string());
        assert!((a - b).is_nan());

        let a = Value::Undefined;
        let b = Value::Boolean(true);
        assert!((a - b).is_nan());

        let a = Value::Undefined;
        let b = Value::Object(Object.into());
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_number_null() {
        let a = Value::Number(1.0);
        let b = Value::Null;
        assert_eq!(a - b, Value::Number(1.0));
    }

    #[test]
    fn sub_number_undefined() {
        let a = Value::Number(1.0);
        let b = Value::Undefined;
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_numbers_number() {
        let a = Value::Number(1.0);
        let b = Value::Number(2.0);
        assert_eq!(a - b, Value::Number(-1.0));
    }

    #[test]
    fn sub_numbers_string() {
        let a = Value::Number(1.0);
        let b = Value::String("2".to_string());
        assert_eq!(a - b, Value::Number(-1.0));

        let a = Value::Number(1.0);
        let b = Value::String("a".to_string());
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_number_boolean() {
        let a = Value::Number(1.0);
        let b = Value::Boolean(true);
        assert_eq!(a - b, Value::Number(0.0));

        let a = Value::Number(1.0);
        let b = Value::Boolean(false);
        assert_eq!(a - b, Value::Number(1.0));
    }

    #[test]
    fn sub_number_object() {
        let a = Value::Number(1.0);
        let b = Value::Object(Object.into());
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_string_null() {
        let a = Value::String("2".to_string());
        let b = Value::Null;
        assert_eq!(a - b, Value::Number(2.0));

        let a = Value::String("a".to_string());
        let b = Value::Null;
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_string_undefined() {
        let a = Value::String("2".to_string());
        let b = Value::Undefined;
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_string_number() {
        let a = Value::String("2".to_string());
        let b = Value::Number(1.0);
        assert_eq!(a - b, Value::Number(1.0));

        let a = Value::String("a".to_string());
        let b = Value::Number(1.0);
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_string_string() {
        let a = Value::String("2".to_string());
        let b = Value::String("1".to_string());
        assert_eq!(a - b, Value::Number(1.0));

        let a = Value::String("a".to_string());
        let b = Value::String("1".to_string());
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_string_boolean() {
        let a = Value::String("2".to_string());
        let b = Value::Boolean(true);
        assert_eq!(a - b, Value::Number(1.0));

        let a = Value::String("a".to_string());
        let b = Value::Boolean(true);
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_string_object() {
        let a = Value::String("2".to_string());
        let b = Value::Object(Object.into());
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_boolean_null() {
        let a = Value::Boolean(true);
        let b = Value::Null;
        assert_eq!(a - b, Value::Number(1.0));

        let a = Value::Boolean(false);
        let b = Value::Null;
        assert_eq!(a - b, Value::Number(0.0));
    }

    #[test]
    fn sub_boolean_undefined() {
        let a = Value::Boolean(true);
        let b = Value::Undefined;
        assert!((a - b).is_nan());

        let a = Value::Boolean(false);
        let b = Value::Undefined;
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_boolean_number() {
        let a = Value::Boolean(true);
        let b = Value::Number(1.0);
        assert_eq!(a - b, Value::Number(0.0));

        let a = Value::Boolean(false);
        let b = Value::Number(1.0);
        assert_eq!(a - b, Value::Number(-1.0));
    }

    #[test]
    fn sub_boolean_string() {
        let a = Value::Boolean(true);
        let b = Value::String("1".to_string());
        assert_eq!(a - b, Value::Number(0.0));

        let a = Value::Boolean(false);
        let b = Value::String("1".to_string());
        assert_eq!(a - b, Value::Number(-1.0));

        let a = Value::Boolean(true);
        let b = Value::String("a".to_string());
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_boolean_boolean() {
        let a = Value::Boolean(true);
        let b = Value::Boolean(true);
        assert_eq!(a - b, Value::Number(0.0));

        let a = Value::Boolean(false);
        let b = Value::Boolean(true);
        assert_eq!(a - b, Value::Number(-1.0));
    }

    #[test]
    fn sub_boolean_object() {
        let a = Value::Boolean(true);
        let b = Value::Object(Object.into());
        assert!((a - b).is_nan());
    }

    #[test]
    fn sub_object_any() {
        let a = Value::Object(Object.into());
        let b = Value::Null;
        assert!((a - b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Undefined;
        assert!((a - b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Number(1.0);
        assert!((a - b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::String("1".to_string());
        assert!((a - b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Boolean(true);
        assert!((a - b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Object(Object.into());
        assert!((a - b).is_nan());
    }

    #[test]
    fn mul_null_any() {
        let a = Value::Null;
        let b = Value::Undefined;
        assert!((a * b).is_nan());

        let a = Value::Null;
        let b = Value::Number(1.0);
        assert_eq!(a * b, Value::Number(0.0));

        let a = Value::Null;
        let b = Value::String("1".to_string());
        assert_eq!(a * b, Value::Number(0.0));

        let a = Value::Null;
        let b = Value::String("a".to_string());
        assert!((a * b).is_nan());

        let a = Value::Null;
        let b = Value::Boolean(true);
        assert_eq!(a * b, Value::Number(0.0));

        let a = Value::Null;
        let b = Value::Object(Object.into());
        assert!((a * b).is_nan());
    }

    #[test]
    fn mul_undefined_any() {
        let a = Value::Undefined;
        let b = Value::Number(1.0);
        assert!((a * b).is_nan());

        let a = Value::Undefined;
        let b = Value::String("1".to_string());
        assert!((a * b).is_nan());

        let a = Value::Undefined;
        let b = Value::String("a".to_string());
        assert!((a * b).is_nan());

        let a = Value::Undefined;
        let b = Value::Boolean(true);
        assert!((a * b).is_nan());

        let a = Value::Undefined;
        let b = Value::Object(Object.into());
        assert!((a * b).is_nan());
    }

    #[test]
    fn mul_numbers_number() {
        let a = Value::Number(2.0);
        let b = Value::Number(3.0);
        assert_eq!(a * b, Value::Number(6.0));
    }

    #[test]
    fn mul_numbers_string() {
        let a = Value::Number(2.0);
        let b = Value::String("3".to_string());
        assert_eq!(a * b, Value::Number(6.0));

        let a = Value::Number(2.0);
        let b = Value::String("a".to_string());
        assert!((a * b).is_nan());
    }

    #[test]
    fn mul_number_boolean() {
        let a = Value::Number(2.0);
        let b = Value::Boolean(true);
        assert_eq!(a * b, Value::Number(2.0));

        let a = Value::Number(2.0);
        let b = Value::Boolean(false);
        assert_eq!(a * b, Value::Number(0.0));
    }

    #[test]
    fn mul_number_object() {
        let a = Value::Number(2.0);
        let b = Value::Object(Object.into());
        assert!((a * b).is_nan());
    }

    #[test]
    fn mul_string_string() {
        let a = Value::String("2".to_string());
        let b = Value::String("3".to_string());
        assert_eq!(a * b, Value::Number(6.0));

        let a = Value::String("2".to_string());
        let b = Value::String("a".to_string());
        assert!((a * b).is_nan());
    }

    #[test]
    fn mul_string_boolean() {
        let a = Value::String("2".to_string());
        let b = Value::Boolean(true);
        assert_eq!(a * b, Value::Number(2.0));

        let a = Value::String("2".to_string());
        let b = Value::Boolean(false);
        assert_eq!(a * b, Value::Number(0.0));

        let a = Value::String("a".to_string());
        let b = Value::Boolean(true);
        assert!((a * b).is_nan());
    }

    #[test]
    fn mul_string_object() {
        let a = Value::String("2".to_string());
        let b = Value::Object(Object.into());
        assert!((a * b).is_nan());
    }

    #[test]
    fn mul_boolean_number() {
        let a = Value::Boolean(true);
        let b = Value::Number(2.0);
        assert_eq!(a * b, Value::Number(2.0));

        let a = Value::Boolean(false);
        let b = Value::Number(2.0);
        assert_eq!(a * b, Value::Number(0.0));
    }

    #[test]
    fn mul_boolean_boolean() {
        let a = Value::Boolean(true);
        let b = Value::Boolean(true);
        assert_eq!(a * b, Value::Number(1.0));

        let a = Value::Boolean(false);
        let b = Value::Boolean(true);
        assert_eq!(a * b, Value::Number(0.0));
    }

    #[test]
    fn mul_boolean_object() {
        let a = Value::Boolean(true);
        let b = Value::Object(Object.into());
        assert!((a * b).is_nan());
    }

    #[test]
    fn mul_object_any() {
        let a = Value::Object(Object.into());
        let b = Value::Number(1.0);
        assert!((a * b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::String("1".to_string());
        assert!((a * b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Boolean(true);
        assert!((a * b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Object(Object.into());
        assert!((a * b).is_nan());
    }

    #[test]
    fn div_null_null() {
        let a = Value::Null;
        let b = Value::Null;
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_null_undefined() {
        let a = Value::Null;
        let b = Value::Undefined;
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_null_number() {
        let a = Value::Null;
        let b = Value::Number(2.0);
        assert_eq!(a / b, Value::Number(0.0));
    }

    #[test]
    fn div_null_string() {
        let a = Value::Null;
        let b = Value::String("2".to_string());
        assert_eq!(a / b, Value::Number(0.0));

        let a = Value::Null;
        let b = Value::String("a".to_string());
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_null_boolean() {
        let a = Value::Null;
        let b = Value::Boolean(true);
        assert_eq!(a / b, Value::Number(0.0));

        let a = Value::Null;
        let b = Value::Boolean(false);
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_undefined_any() {
        let a = Value::Undefined;
        let b = Value::Null;
        assert!((a / b).is_nan());

        let a = Value::Undefined;
        let b = Value::Number(1.0);
        assert!((a / b).is_nan());

        let a = Value::Undefined;
        let b = Value::String("1".to_string());
        assert!((a / b).is_nan());

        let a = Value::Undefined;
        let b = Value::Boolean(true);
        assert!((a / b).is_nan());

        let a = Value::Undefined;
        let b = Value::Object(Object.into());
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_number_null() {
        let a = Value::Number(6.0);
        let b = Value::Null;
        assert_eq!(a / b, Value::Number(f64::INFINITY));

        let a = Value::Number(0.0);
        let b = Value::Null;
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_number_number() {
        let a = Value::Number(6.0);
        let b = Value::Number(2.0);
        assert_eq!(a / b, Value::Number(3.0));

        let a = Value::Number(0.0);
        let b = Value::Number(0.0);
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_number_string() {
        let a = Value::Number(6.0);
        let b = Value::String("2".to_string());
        assert_eq!(a / b, Value::Number(3.0));

        let a = Value::Number(6.0);
        let b = Value::String("a".to_string());
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_number_boolean() {
        let a = Value::Number(6.0);
        let b = Value::Boolean(true);
        assert_eq!(a / b, Value::Number(6.0));

        let a = Value::Number(6.0);
        let b = Value::Boolean(false);
        assert_eq!(a / b, Value::Number(f64::INFINITY));
    }

    #[test]
    fn div_string_null() {
        let a = Value::String("6".to_string());
        let b = Value::Null;
        assert_eq!(a / b, Value::Number(f64::INFINITY));

        let a = Value::String("a".to_string());
        let b = Value::Null;
        assert!((a / b).is_nan());

        let a = Value::String("0".to_string());
        let b = Value::Null;
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_string_number() {
        let a = Value::String("6".to_string());
        let b = Value::Number(2.0);
        assert_eq!(a / b, Value::Number(3.0));

        let a = Value::String("a".to_string());
        let b = Value::Number(2.0);
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_string_string() {
        let a = Value::String("6".to_string());
        let b = Value::String("2".to_string());
        assert_eq!(a / b, Value::Number(3.0));

        let a = Value::String("6".to_string());
        let b = Value::String("a".to_string());
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_string_boolean() {
        let a = Value::String("6".to_string());
        let b = Value::Boolean(true);
        assert_eq!(a / b, Value::Number(6.0));

        let a = Value::String("6".to_string());
        let b = Value::Boolean(false);
        assert_eq!(a / b, Value::Number(f64::INFINITY));
    }

    #[test]
    fn div_boolean_number() {
        let a = Value::Boolean(true);
        let b = Value::Number(2.0);
        assert_eq!(a / b, Value::Number(0.5));

        let a = Value::Boolean(false);
        let b = Value::Number(2.0);
        assert_eq!(a / b, Value::Number(0.0));
    }

    #[test]
    fn div_boolean_boolean() {
        let a = Value::Boolean(true);
        let b = Value::Boolean(true);
        assert_eq!(a / b, Value::Number(1.0));

        let a = Value::Boolean(true);
        let b = Value::Boolean(false);
        assert_eq!(a / b, Value::Number(f64::INFINITY));

        let a = Value::Boolean(false);
        let b = Value::Boolean(false);
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_object_any() {
        let a = Value::Object(Object.into());
        let b = Value::Number(1.0);
        assert!((a / b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::String("1".to_string());
        assert!((a / b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Boolean(true);
        assert!((a / b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Object(Object.into());
        assert!((a / b).is_nan());
    }

    #[test]
    fn div_any_undefined() {
        let a = Value::Null;
        let b = Value::Undefined;
        assert!((a / b).is_nan());

        let a = Value::Number(1.0);
        let b = Value::Undefined;
        assert!((a / b).is_nan());

        let a = Value::String("1".to_string());
        let b = Value::Undefined;
        assert!((a / b).is_nan());

        let a = Value::Boolean(true);
        let b = Value::Undefined;
        assert!((a / b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Undefined;
        assert!((a / b).is_nan());
    }

    #[test]
    fn rem_null_any() {
        let a = Value::Null;
        let b = Value::Null;
        assert!((a % b).is_nan());

        let a = Value::Null;
        let b = Value::Undefined;
        assert!((a % b).is_nan());

        let a = Value::Null;
        let b = Value::Number(1.0);
        assert_eq!(a % b, Value::Number(0.0));

        let a = Value::Null;
        let b = Value::String("1".to_string());
        assert_eq!(a % b, Value::Number(0.0));

        let a = Value::Null;
        let b = Value::String("a".to_string());
        assert!((a % b).is_nan());

        let a = Value::Null;
        let b = Value::Boolean(true);
        assert_eq!(a % b, Value::Number(0.0));

        let a = Value::Null;
        let b = Value::Object(Object.into());
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_any_null() {
        let a = Value::Number(1.0);
        let b = Value::Null;
        assert!((a % b).is_nan());

        let a = Value::Undefined;
        let b = Value::Null;
        assert!((a % b).is_nan());

        let a = Value::String("1".to_string());
        let b = Value::Null;
        assert!((a % b).is_nan());

        let a = Value::String("a".to_string());
        let b = Value::Null;
        assert!((a % b).is_nan());

        let a = Value::Boolean(true);
        let b = Value::Null;
        assert!((a % b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Null;
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_undefined_any() {
        let a = Value::Undefined;
        let b = Value::Null;
        assert!((a % b).is_nan());

        let a = Value::Undefined;
        let b = Value::Undefined;
        assert!((a % b).is_nan());

        let a = Value::Undefined;
        let b = Value::Number(1.0);
        assert!((a % b).is_nan());

        let a = Value::Undefined;
        let b = Value::String("1".to_string());
        assert!((a % b).is_nan());

        let a = Value::Undefined;
        let b = Value::String("a".to_string());
        assert!((a % b).is_nan());

        let a = Value::Undefined;
        let b = Value::Boolean(true);
        assert!((a % b).is_nan());

        let a = Value::Undefined;
        let b = Value::Object(Object.into());
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_any_undefined() {
        let a = Value::Null;
        let b = Value::Undefined;
        assert!((a % b).is_nan());

        let a = Value::Number(1.0);
        let b = Value::Undefined;
        assert!((a % b).is_nan());

        let a = Value::String("1".to_string());
        let b = Value::Undefined;
        assert!((a % b).is_nan());

        let a = Value::Boolean(true);
        let b = Value::Undefined;
        assert!((a % b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Undefined;
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_number_number() {
        let a = Value::Number(10.0);
        let b = Value::Number(3.0);
        assert_eq!(a % b, Value::Number(1.0));
    }

    #[test]
    fn rem_number_string() {
        let a = Value::Number(10.0);
        let b = Value::String("3".to_string());
        assert_eq!(a % b, Value::Number(1.0));

        let a = Value::Number(10.0);
        let b = Value::String("a".to_string());
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_string_number() {
        let a = Value::String("10".to_string());
        let b = Value::Number(3.0);
        assert_eq!(a % b, Value::Number(1.0));

        let a = Value::String("a".to_string());
        let b = Value::Number(3.0);
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_string_boolean() {
        let a = Value::String("10".to_string());
        let b = Value::Boolean(true);
        assert_eq!(a % b, Value::Number(0.0));

        let a = Value::String("a".to_string());
        let b = Value::Boolean(true);
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_boolean_number() {
        let a = Value::Boolean(true);
        let b = Value::Number(2.0);
        assert_eq!(a % b, Value::Number(1.0));

        let a = Value::Boolean(false);
        let b = Value::Number(2.0);
        assert_eq!(a % b, Value::Number(0.0));
    }

    #[test]
    fn rem_boolean_string() {
        let a = Value::Boolean(true);
        let b = Value::String("2".to_string());
        assert_eq!(a % b, Value::Number(1.0));

        let a = Value::Boolean(true);
        let b = Value::String("a".to_string());
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_boolean_boolean() {
        let a = Value::Boolean(true);
        let b = Value::Boolean(true);
        assert_eq!(a % b, Value::Number(0.0));

        let a = Value::Boolean(true);
        let b = Value::Boolean(false);
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_object_any() {
        let a = Value::Object(Object.into());
        let b = Value::Null;
        assert!((a % b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Undefined;
        assert!((a % b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Number(1.0);
        assert!((a % b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::String("1".to_string());
        assert!((a % b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Boolean(true);
        assert!((a % b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Object(Object.into());
        assert!((a % b).is_nan());
    }

    #[test]
    fn rem_any_object() {
        let a = Value::Null;
        let b = Value::Object(Object.into());
        assert!((a % b).is_nan());

        let a = Value::Number(1.0);
        let b = Value::Object(Object.into());
        assert!((a % b).is_nan());

        let a = Value::String("1".to_string());
        let b = Value::Object(Object.into());
        assert!((a % b).is_nan());

        let a = Value::Boolean(true);
        let b = Value::Object(Object.into());
        assert!((a % b).is_nan());

        let a = Value::Object(Object.into());
        let b = Value::Object(Object.into());
        assert!((a % b).is_nan());
    }

    #[test]
    fn null_equals_null() {
        let a = Value::Null;
        let b = Value::Null;
        assert!(a >= b);
        assert!(a <= b);
        assert_eq!(a, b);
    }

    #[test]
    fn null_less_than_number() {
        let a = Value::Null;
        let b = Value::Number(1.0);
        assert!(a < b);
        assert!(a <= b);
        assert_ne!(a, b);
    }

    #[test]
    fn null_less_than_boolean() {
        let a = Value::Null;
        let b = Value::Boolean(true);
        assert!(a < b);
        assert!(a <= b);
        assert_ne!(a, b);
    }

    #[test]
    fn number_greater_than_null() {
        let a = Value::Number(1.0);
        let b = Value::Null;
        assert!(a > b);
        assert!(a >= b);
        assert_ne!(a, b);
    }

    #[test]
    fn number_equals_itself() {
        let a = Value::Number(1.0);
        let b = Value::Number(1.0);
        assert!(a >= b);
        assert!(a <= b);
        assert_eq!(a, b);
    }

    #[test]
    fn number_less_than_greater_number() {
        let a = Value::Number(1.0);
        let b = Value::Number(2.0);
        assert!(a < b);
        assert!(a <= b);
        assert_ne!(a, b);
    }

    #[test]
    fn string_greater_than_null() {
        let a = Value::String("1".to_string());
        let b = Value::Null;
        assert!(a > b);
        assert!(a >= b);
        assert_ne!(a, b);
    }

    #[test]
    fn string_equals_itself() {
        let a = Value::String("1".to_string());
        let b = Value::String("1".to_string());
        assert!(a >= b);
        assert!(a <= b);
        assert_eq!(a, b);
    }

    #[test]
    fn string_less_than_greater_string() {
        let a = Value::String("1".to_string());
        let b = Value::String("2".to_string());
        assert!(a < b);
        assert!(a <= b);
        assert_ne!(a, b);
    }

    #[test]
    fn boolean_greater_than_null() {
        let a = Value::Boolean(true);
        let b = Value::Null;
        assert!(a > b);
        assert!(a >= b);
        assert_ne!(a, b);
    }

    #[test]
    fn boolean_equals_itself() {
        let a = Value::Boolean(true);
        let b = Value::Boolean(true);
        assert!(a >= b);
        assert!(a <= b);
        assert_eq!(a, b);
    }

    #[test]
    fn boolean_less_than_greater_boolean() {
        let a = Value::Boolean(false);
        let b = Value::Boolean(true);
        assert!(a < b);
        assert!(a <= b);
        assert_ne!(a, b);
    }

    #[test]
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn object_not_comparable() {
        let a = Value::Object(Object.into());
        let b = Value::Null;
        assert!(!(a >= b));
        assert!(!(a <= b));
        assert!(!(a == b));
        assert!(!(a > b));
        assert!(!(a < b));
        assert_ne!(a, b);
    }

    #[test]
    fn shl_number_number() {
        let a = Value::Number(10.0);
        let b = Value::Number(2.0);
        assert_eq!(a << b, Value::Number(40.0));
    }

    #[test]
    fn shl_float_number() {
        let a = Value::Number(10.5);
        let b = Value::Number(2.0);
        assert_eq!(a << b, Value::Number(40.0));
    }

    #[test]
    fn shl_number_float() {
        let a = Value::Number(10.0);
        let b = Value::Number(2.5);
        assert_eq!(a << b, Value::Number(40.0));
    }

    #[test]
    fn shl_null_any() {
        let a = Value::Null;
        let b = Value::Number(2.0);
        assert_eq!(a << b, Value::Number(0.0));

        let a = Value::Null;
        let b = Value::Null;
        assert_eq!(a << b, Value::Number(0.0));
    }

    #[test]
    fn shl_any_null() {
        let a = Value::Number(10.0);
        let b = Value::Null;
        assert_eq!(a << b, Value::Number(10.0));
    }

    #[test]
    fn shl_undefined_any() {
        let a = Value::Undefined;
        let b = Value::Number(2.0);
        assert_eq!(a << b, Value::Number(0.0));

        let a = Value::Undefined;
        let b = Value::Undefined;
        assert_eq!(a << b, Value::Number(0.0));
    }

    #[test]
    fn shl_any_undefined() {
        let a = Value::Number(10.0);
        let b = Value::Undefined;
        assert_eq!(a << b, Value::Number(10.0));
    }

    #[test]
    fn shl_string_number() {
        let a = Value::String("10".to_string());
        let b = Value::Number(2.0);
        assert_eq!(a << b, Value::Number(40.0));
    }

    #[test]
    fn shl_number_string() {
        let a = Value::Number(10.0);
        let b = Value::String("2".to_string());
        assert_eq!(a << b, Value::Number(40.0));
    }

    #[test]
    fn shl_boolean_number() {
        let a = Value::Boolean(true);
        let b = Value::Number(2.0);
        assert_eq!(a << b, Value::Number(4.0));
    }

    #[test]
    fn shl_number_boolean() {
        let a = Value::Number(10.0);
        let b = Value::Boolean(true);
        assert_eq!(a << b, Value::Number(20.0));
    }

    #[test]
    fn shl_object_any() {
        let a = Value::Object(Object.into());
        let b = Value::Number(2.0);
        assert_eq!(a << b, Value::Number(0.0));

        let a = Value::Object(Object.into());
        let b = Value::String("2".to_string());
        assert_eq!(a << b, Value::Number(0.0));
    }

    #[test]
    fn shl_any_object() {
        let a = Value::Number(10.0);
        let b = Value::Object(Object.into());
        assert_eq!(a << b, Value::Number(10.0));

        let a = Value::String("10".to_string());
        let b = Value::Object(Object.into());
        assert_eq!(a << b, Value::Number(10.0));
    }

    #[test]
    fn shr_number_number() {
        let a = Value::Number(10.0);
        let b = Value::Number(2.0);
        assert_eq!(a >> b, Value::Number(2.0));
    }

    #[test]
    fn shr_null_any() {
        let a = Value::Null;
        let b = Value::Number(2.0);
        assert_eq!(a >> b, Value::Number(0.0));
    }

    #[test]
    fn shr_any_null() {
        let a = Value::Number(10.0);
        let b = Value::Null;
        assert_eq!(a >> b, Value::Number(10.0));
    }

    #[test]
    fn shr_undefined_any() {
        let a = Value::Undefined;
        let b = Value::Number(2.0);
        assert_eq!(a >> b, Value::Number(0.0));
    }

    #[test]
    fn shr_any_undefined() {
        let a = Value::Number(10.0);
        let b = Value::Undefined;
        assert_eq!(a >> b, Value::Number(10.0));
    }

    #[test]
    fn shr_string_number() {
        let a = Value::String("10".to_string());
        let b = Value::Number(2.0);
        assert_eq!(a >> b, Value::Number(2.0));

        let a = Value::String("a".to_string());
        let b = Value::Number(2.0);
        assert_eq!(a >> b, Value::Number(0.0));
    }

    #[test]
    fn shr_number_string() {
        let a = Value::Number(10.0);
        let b = Value::String("2".to_string());
        assert_eq!(a >> b, Value::Number(2.0));

        let a = Value::Number(10.0);
        let b = Value::String("a".to_string());
        assert_eq!(a >> b, Value::Number(10.0));
    }

    #[test]
    fn shr_boolean_number() {
        let a = Value::Boolean(true);
        let b = Value::Number(2.0);
        assert_eq!(a >> b, Value::Number(0.0));
    }

    #[test]
    fn shr_number_boolean() {
        let a = Value::Number(10.0);
        let b = Value::Boolean(true);
        assert_eq!(a >> b, Value::Number(5.0));
    }

    #[test]
    fn shr_object_any() {
        let a = Value::Object(Object.into());
        let b = Value::Number(2.0);
        assert_eq!(a >> b, Value::Number(0.0));
    }

    #[test]
    fn shr_any_object() {
        let a = Value::Number(10.0);
        let b = Value::Object(Object.into());
        assert_eq!(a >> b, Value::Number(10.0));
    }

    #[test]
    fn bit_or() {
        assert_eq!(
            Value::Number(10.0) | Value::Number(2.0),
            Value::Number(f64::from(10 | 2))
        );
        assert_eq!(
            Value::Number(10.0) | Value::Boolean(true),
            Value::Number(f64::from(10 | 1))
        );
        assert_eq!(
            Value::String("10".to_string()) | Value::Number(2.0),
            Value::Number(f64::from(10 | 2))
        );
        assert_eq!(
            Value::String("invalid".to_string()) | Value::Number(2.0),
            Value::Number(2.0)
        );
        assert_eq!(
            Value::Object(Object.into()) | Value::Number(2.0),
            Value::Number(2.0)
        );
    }

    #[test]
    fn bit_and() {
        assert_eq!(
            Value::Number(10.0) & Value::Number(2.0),
            Value::Number(f64::from(10 & 2))
        );
        assert_eq!(
            Value::Number(10.0) & Value::Boolean(true),
            Value::Number(f64::from(10 & 1))
        );
        assert_eq!(
            Value::String("10".to_string()) & Value::Number(2.0),
            Value::Number(f64::from(10 & 2))
        );
        assert_eq!(
            Value::String("invalid".to_string()) & Value::Number(2.0),
            Value::Number(0.0)
        );
        assert_eq!(
            Value::Object(Object.into()) & Value::Number(2.0),
            Value::Number(0.0)
        );
    }

    #[test]
    fn bit_xor() {
        assert_eq!(
            Value::Number(10.0) ^ Value::Number(2.0),
            Value::Number(f64::from(10 ^ 2))
        );
        assert_eq!(
            Value::Number(10.0) ^ Value::Boolean(true),
            Value::Number(f64::from(10 ^ 1))
        );
        assert_eq!(
            Value::String("10".to_string()) ^ Value::Number(2.0),
            Value::Number(f64::from(10 ^ 2))
        );
        assert_eq!(
            Value::String("invalid".to_string()) ^ Value::Number(2.0),
            Value::Number(2.0)
        );
        assert_eq!(
            Value::Object(Object.into()) ^ Value::Number(2.0),
            Value::Number(2.0)
        );
    }
}
