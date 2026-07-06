#![allow(clippy::match_same_arms)]

mod add;
mod and;
mod div;
mod exp;
mod mul;
mod or;
mod rem;
mod shl;
mod shr;
mod sub;
mod ushr;
mod xor;

use super::{Hint, Value};
use crate::error::Error;
use crate::{ObjectOrNull, Realm};
use num_bigint::BigInt;
use num_traits::{FromPrimitive, Num, One, ToPrimitive, Zero};
use std::cmp::Ordering;
use std::rc::Rc;
use std::str::FromStr;

pub trait ToNumber {
    fn num(&self) -> f64;
}

impl ToNumber for bool {
    fn num(&self) -> f64 {
        if *self { 1.0 } else { 0.0 }
    }
}

impl ToNumber for &str {
    fn num(&self) -> f64 {
        // Trim leading and trailing whitespace (JavaScript semantics)
        let trimmed = self.trim();

        if trimmed.is_empty() {
            0.0
        } else {
            if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                if !only_numeric_digits(&trimmed[2..], 16) {
                    return f64::NAN;
                }

                return f64::from_str_radix(&trimmed[2..], 16).unwrap_or(f64::NAN);
            }

            if trimmed.starts_with("0b") || trimmed.starts_with("0B") {
                if !only_numeric_digits(&trimmed[2..], 2) {
                    return f64::NAN;
                }

                return f64::from_str_radix(&trimmed[2..], 2).unwrap_or(f64::NAN);
            }

            if trimmed.starts_with("0o") || trimmed.starts_with("0O") {
                if !only_numeric_digits(&trimmed[2..], 8) {
                    return f64::NAN;
                }

                return f64::from_str_radix(&trimmed[2..], 8).unwrap_or(f64::NAN);
            }

            if trimmed == "Infinity" || trimmed == "+Infinity" {
                return f64::INFINITY;
            }
            if trimmed == "-Infinity" {
                return f64::NEG_INFINITY;
            }

            let lower = trimmed.to_lowercase();
            if lower == "infinity"
                || lower == "+infinity"
                || lower == "-infinity"
                || lower == "inf"
                || lower == "+inf"
                || lower == "-inf"
            {
                return f64::NAN;
            }

            trimmed.parse().unwrap_or(f64::NAN)
        }
    }
}

fn only_numeric_digits(s: &str, radix: u32) -> bool {
    s.chars().all(|c| c.is_digit(radix))
}

pub enum BigIntOrNumber {
    BigInt(Rc<BigInt>),
    Number(f64),
}

impl BigIntOrNumber {
    #[must_use]
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Self::BigInt(b) => b.to_f64(),
            Self::Number(n) => Some(*n),
        }
    }

    pub fn to_big_int(&self) -> Option<Rc<BigInt>> {
        match self {
            Self::BigInt(b) => Some(Rc::clone(b)),
            Self::Number(n) => BigInt::from_f64(*n).map(Rc::new),
        }
    }
}

impl Value {
    #[must_use]
    pub fn to_number_or_null(&self) -> f64 {
        match self {
            Self::Number(n) => *n,
            Self::Boolean(b) => b.num(),
            Self::String(s) => s.parse().unwrap_or(0.0),
            Self::BigInt(b) => b.to_f64().unwrap_or(f64::NAN),
            _ => 0.0,
        }
    }

    pub fn to_numeric(&self, realm: &mut Realm) -> Result<BigIntOrNumber, Error> {
        Ok(BigIntOrNumber::Number(match self {
            Self::Number(n) => *n,
            Self::Undefined => f64::NAN,
            Self::Null => 0.0,
            Self::Boolean(b) => b.num(),
            Self::String(s) => (&*s.as_str_lossy()).num(),
            Self::Object(_) => {
                let v = self.to_primitive(Hint::Number, realm)?.assert_no_object()?;

                return v.to_numeric(realm);
            }
            Self::BigInt(b) => return Ok(BigIntOrNumber::BigInt(Rc::clone(b))),
            Self::Symbol(_) => return Err(Error::ty("Cannot convert Symbol to numeric value")),
        }))
    }
    pub fn to_number(&self, realm: &mut Realm) -> Result<f64, Error> {
        Ok(match self {
            Self::Number(n) => *n,
            Self::Undefined => f64::NAN,
            Self::Null => 0.0,
            Self::Boolean(b) => b.num(),
            Self::String(s) => (&*s.as_str_lossy()).num(),
            Self::Object(_) => {
                let v = self.to_primitive(Hint::Number, realm)?.assert_no_object()?;

                return v.to_number(realm);
            }
            Self::Symbol(_) | Self::BigInt(_) => {
                return Err(Error::ty("Cannot convert BigInt or Symbol to number"));
            }
        })
    }

    pub fn to_big_int(&self, realm: &mut Realm) -> Result<BigInt, Error> {
        // 1. Let prim be ? ToPrimitive(argument, number).
        let prim = self.to_primitive(Hint::Number, realm)?.assert_no_object()?;

        // 2. Based on the type of prim:
        Ok(match prim {
            // Number -> NumberToBigInt(prim) - throws RangeError for non-integers
            Self::Number(n) => number_to_big_int(n)?,
            Self::Undefined => return Err(Error::ty("Cannot convert undefined to BigInt")),
            Self::Null => return Err(Error::ty("Cannot convert null to BigInt")),
            Self::Boolean(b) => {
                if b {
                    BigInt::one()
                } else {
                    BigInt::zero()
                }
            }
            // String -> StringToBigInt(prim) - throws SyntaxError for invalid strings
            Self::String(s) => parse_big_int(&s.as_str_lossy())?,
            Self::BigInt(b) => (*b).clone(),
            Self::Symbol(_) => return Err(Error::ty("Cannot convert Symbol to BigInt")),
            Self::Object(_) => return Err(Error::new("ToPrimitive should have converted object")),
        })
    }

    /// ToBigInt abstract operation - throws TypeError for Number values.
    /// This is different from `to_big_int` which is used by the BigInt constructor
    /// and converts integral numbers using NumberToBigInt.
    pub fn to_big_int_strict(&self, realm: &mut Realm) -> Result<BigInt, Error> {
        // 1. Let prim be ? ToPrimitive(argument, number).
        let prim = self.to_primitive(Hint::Number, realm)?.assert_no_object()?;

        // 2. Return the value that prim corresponds to in the ToBigInt table.
        Ok(match prim {
            Self::Undefined => return Err(Error::ty("Cannot convert undefined to BigInt")),
            Self::Null => return Err(Error::ty("Cannot convert null to BigInt")),
            Self::Boolean(b) => {
                if b {
                    BigInt::one()
                } else {
                    BigInt::zero()
                }
            }
            Self::BigInt(b) => (*b).clone(),
            Self::Number(_) => return Err(Error::ty("Cannot convert Number to BigInt")),
            Self::String(s) => parse_big_int(&s.as_str_lossy())
                .map_err(|_| Error::syn_error(format!("Cannot convert '{s}' to BigInt")))?,
            Self::Symbol(_) => return Err(Error::ty("Cannot convert Symbol to BigInt")),
            Self::Object(_) => return Err(Error::new("ToPrimitive should have converted object")),
        })
    }

    pub fn to_int_or_null(&self, realm: &mut Realm) -> Result<i64, Error> {
        Ok(match self {
            Self::Number(n) => *n as i64,
            Self::Boolean(b) => i64::from(*b),
            Self::String(s) => s.parse().unwrap_or(0),
            Self::Object(o) => o
                .to_primitive(Hint::Number, realm)?
                .assert_no_object()?
                .to_int_or_null(realm)?,
            Self::Symbol(_) => return Err(Error::ty("Cannot convert Symbol to number")),
            _ => 0,
        })
    }
}

/// NumberToBigInt ( number ) - per ECMAScript spec
/// Throws RangeError if the number is not a safe integer (NaN, Infinity, or has fractional part)
pub fn number_to_big_int(n: f64) -> Result<BigInt, Error> {
    // 1. If IsIntegralNumber(number) is false, throw a RangeError exception.
    // IsIntegralNumber returns false for NaN, Infinity, -Infinity, and non-integers
    if n.is_nan() {
        return Err(Error::range("Cannot convert NaN to BigInt"));
    }
    if n.is_infinite() {
        return Err(Error::range("Cannot convert Infinity to BigInt"));
    }
    if n.fract() != 0.0 {
        return Err(Error::range("Cannot convert non-integer to BigInt"));
    }

    // 2. Return ℤ(ℝ(number)).
    BigInt::from_f64(n).ok_or_else(|| Error::range("Cannot convert number to BigInt"))
}

/// StringToBigInt - per ECMAScript spec
/// Returns None if the string cannot be parsed as a BigInt
fn string_to_big_int(s: &str) -> Option<BigInt> {
    let s = s.trim();

    if s.is_empty() {
        return Some(BigInt::zero());
    }

    // Check for negative hex/octal/binary which is not allowed
    if s.starts_with("-0x")
        || s.starts_with("-0X")
        || s.starts_with("-0b")
        || s.starts_with("-0B")
        || s.starts_with("-0o")
        || s.starts_with("-0O")
    {
        return None;
    }

    if s.starts_with("0x") || s.starts_with("0X") {
        let digits = &s[2..];
        if digits.is_empty() {
            return None;
        }
        return BigInt::from_str_radix(digits, 16).ok();
    }

    if s.starts_with("0b") || s.starts_with("0B") {
        let digits = &s[2..];
        if digits.is_empty() {
            return None;
        }
        return BigInt::from_str_radix(digits, 2).ok();
    }

    if s.starts_with("0o") || s.starts_with("0O") {
        let digits = &s[2..];
        if digits.is_empty() {
            return None;
        }
        return BigInt::from_str_radix(digits, 8).ok();
    }

    BigInt::from_str(s).ok()
}

fn parse_big_int(s: &str) -> Result<BigInt, Error> {
    string_to_big_int(s).ok_or_else(|| Error::syn_error(format!("Cannot convert '{s}' to BigInt")))
}

impl PartialOrd for Value {
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
            (Self::String(a), Self::String(b)) => a
                .as_str_lossy()
                .as_ref()
                .partial_cmp(b.as_str_lossy().as_ref()),
            (Self::String(a), Self::Boolean(b)) => {
                let a = a.parse::<f64>().ok()?;
                a.partial_cmp(&b.num())
            }
            (Self::String(a), Self::Object(_)) => {
                if &*a.as_str_lossy() == "[object Object]" {
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

            (Self::BigInt(a), Self::BigInt(b)) => a.partial_cmp(b),

            (Self::Number(a), Self::BigInt(b)) => a.partial_cmp(&b.to_f64().unwrap_or(f64::NAN)),
            (Self::BigInt(a), Self::Number(b)) => a.to_f64().unwrap_or(f64::NAN).partial_cmp(b),

            (Self::String(a), Self::BigInt(b)) | (Self::BigInt(b), Self::String(a)) => {
                let a = a.parse().unwrap_or(BigInt::zero());

                a.partial_cmp(b)
            }

            (_, _) => None,
        }
    }
}

impl Value {
    #[must_use]
    pub fn log_or(&self, rhs: Self) -> Self {
        if self.is_truthy() { self.copy() } else { rhs }
    }

    #[must_use]
    pub fn log_and(&self, rhs: Self) -> Self {
        if self.is_truthy() { rhs } else { self.copy() }
    }

    // pub fn pow(&self, rhs: &Self, realm: &mut Realm) -> Result<Self, Error> {
    //     if let (Self::BigInt(a), Self::BigInt(b)) = (self, rhs) {
    //         return Ok(Self::BigInt(a.pow(b.to_u32().unwrap_or(0))));
    //     }
    //
    //     Ok(Self::Number(
    //         self.to_number(realm)?.powf(rhs.to_number(realm)?),
    //     ))
    // }

    pub fn normal_eq(&self, rhs: &Self, realm: &mut Realm) -> Result<bool, Error> {
        match (self, rhs) {
            (Self::Object(lhs), Self::Object(rhs)) => {
                return Ok(lhs == rhs);
            }
            (Self::Object(_), Self::Undefined | Self::Null) |
            (Self::Undefined | Self::Null, Self::Object(_)) => {
                return Ok(false);
            }

            (Self::Null | Self::Undefined, Self::Null | Self::Undefined) => return Ok(true),
            (Self::Number(a), Self::Number(b)) => return Ok(a == b),
            (Self::String(a), Self::String(b)) => return Ok(a == b),
            (Self::Boolean(a), Self::Boolean(b)) => return Ok(a == b),
            (Self::Symbol(a), Self::Symbol(b)) => return Ok(a == b),

            _ => {}

        }


        let lhs = self.to_primitive(Hint::None, realm)?;
        let rhs = rhs.to_primitive(Hint::None, realm)?;

        Ok(match (lhs, rhs) {
            (Self::Null | Self::Undefined, Self::Null | Self::Undefined) => true,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => a == b,
            (Self::Symbol(a), Self::Symbol(b)) => a == b,

            (Self::Number(a), Self::String(b)) | (Self::String(b), Self::Number(a)) => {
                if a == 0.0 && b.is_empty() {
                    return Ok(true);
                }

                a.to_string() == b.to_string()
            }

            (Self::Number(a), Self::Boolean(b)) | (Self::Boolean(b), Self::Number(a)) => {
                a == b.num()
            }

            (Self::Number(a), Self::Object(b)) | (Self::Object(b), Self::Number(a)) => {
                let b = format!("{b}");

                if a == 0.0 && b.is_empty() {
                    return Ok(true);
                }

                a.to_string() == b
            }

            (Self::String(a), Self::Object(b)) | (Self::Object(b), Self::String(a)) => {
                a.to_string() == format!("{b}")
            }

            (Self::String(a), Self::Boolean(b)) | (Self::Boolean(b), Self::String(a)) => {
                (a.is_empty() && !b) || a.to_string() == b.num().to_string()
            }

            (Self::Boolean(a), Self::Object(b)) | (Self::Object(b), Self::Boolean(a)) => {
                a.num().to_string() == format!("{b}")
            }

            (Self::BigInt(a), Self::BigInt(b)) => a == b,

            (Self::BigInt(a), Self::Number(b)) | (Self::Number(b), Self::BigInt(a)) => {
                a.to_f64().unwrap_or(f64::NAN) == b
            }

            (Self::BigInt(a), Self::String(b)) | (Self::String(b), Self::BigInt(a)) => {
                a.to_string() == b.to_string()
            }

            _ => false,
        })
    }

    pub fn instance_of(&self, rhs: &Self, realm: &mut Realm) -> Result<bool, Error> {
        let Self::Object(obj) = self else {
            return Ok(false);
        };

        let Self::Object(constructor) = rhs else {
            return Err(Error::ty(
                "Right-hand side of 'instanceof' is not an object",
            ));
        };

        //TODO: this is kinda a hack, but should always work
        let Self::Object(constructor_proto) = constructor
            .resolve_property("prototype", realm)?
            .ok_or(Error::ty(
                "Right-hand side of 'instanceof' is not a constructor",
            ))?
        else {
            return Err(Error::ty(
                "Right-hand side of 'instanceof' has not an object as constructor",
            ));
        };

        let constructor_proto = ObjectOrNull::Object(constructor_proto.into());

        let mut proto = Some(obj.prototype(realm)?);

        while let Some(p) = proto {
            if p == constructor_proto {
                return Ok(true);
            }

            if let ObjectOrNull::Object(o) = p {
                proto = Some(o.prototype(realm)?);
            } else {
                break;
            }
        }

        Ok(false)
    }

    pub fn is_proto_cycle(&self, rhs: Self, realm: &mut Realm) -> Result<bool, Error> {
        let Self::Object(obj) = self else {
            return Ok(false);
        };

        let Self::Object(proto_obj) = rhs else {
            return Ok(false);
        };

        let mut proto = Some(obj.prototype(realm)?);
        let proto_obj = ObjectOrNull::Object(proto_obj);

        while let Some(p) = proto {
            if p == proto_obj {
                return Ok(true);
            }

            if let ObjectOrNull::Object(o) = p {
                proto = Some(o.prototype(realm)?);
            } else {
                break;
            }
        }

        Ok(false)
    }
}

