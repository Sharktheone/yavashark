use crate::utils::ProtoDefault;
use crate::value::{fmt_num, Constructor, Func, Obj};
use crate::{MutObject, NativeFunction, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use num_bigint::Sign;
use num_traits::ToPrimitive;
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_string::YSString;

#[object]
#[derive(Debug)]
pub struct NumberObj {
    #[mutable]
    #[primitive]
    number: f64,
}

impl ProtoDefault for NumberObj {
    fn proto_default(realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableNumberObj {
                object: MutObject::with_proto(realm.intrinsics.number.clone()),
                number: 0.0,
            }),
        }
    }

    fn null_proto_default() -> Self {
        Self {
            inner: RefCell::new(MutableNumberObj {
                object: MutObject::null(),
                number: 0.0,
            }),
        }
    }
}

#[object(constructor, function, to_string)]
#[derive(Debug)]
pub struct NumberConstructor {}

impl NumberConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: ObjectHandle, realm: &mut Realm) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableNumberConstructor {
                object: MutObject::with_proto(func.clone()),
            }),
        };

        this.initialize(func, realm)?;

        Ok(this.into_object())
    }

    pub fn override_to_string(&self, _: &mut Realm) -> Res<YSString> {
        Ok("function Number() { [native code] }".into())
    }

    pub fn override_to_string_internal(&self) -> Res<YSString> {
        Ok("function Number() { [native code] }".into())
    }

    fn construct_from(realm: &mut Realm, val: &Value) -> Res<f64> {
        Ok(match val {
            Value::BigInt(v) => v.to_f64().unwrap_or_else(|| {
                let (sign, digits) = v.to_u64_digits();

                let val = digits.first().unwrap_or(&0).to_f64().unwrap_or(0.0);

                if sign == Sign::Minus {
                    -val
                } else {
                    val
                }
            }),
            _ => val.to_number(realm)?,
        })
    }
}

#[properties_new(raw)]
impl NumberConstructor {
    pub const EPSILON: f64 = f64::EPSILON;
    pub const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    pub const MIN_SAFE_INTEGER: f64 = -9_007_199_254_740_991.0;

    pub const MAX_VALUE: f64 = f64::MAX;
    pub const MIN_VALUE: f64 = 5e-324;

    pub const NEGATIVE_INFINITY: f64 = f64::NEG_INFINITY;
    pub const POSITIVE_INFINITY: f64 = f64::INFINITY;
    #[prop("NaN")]
    pub const NAN: f64 = f64::NAN;

    #[prop("isFinite")]
    #[must_use]
    pub const fn is_finite(number: &Value) -> bool {
        if let Value::Number(number) = number {
            number.is_finite()
        } else {
            false
        }
    }

    #[prop("isNaN")]
    #[must_use]
    pub const fn is_nan(number: &Value) -> bool {
        if let Value::Number(number) = number {
            number.is_nan()
        } else {
            false
        }
    }

    #[prop("isInteger")]
    #[must_use]
    pub fn is_integer(number: &Value) -> bool {
        if let Value::Number(number) = number {
            number.fract() == 0.0
        } else {
            false
        }
    }

    #[prop("isSafeInteger")]
    #[must_use]
    pub fn is_safe_integer(number: &Value) -> bool {
        if let Value::Number(number) = number {
            number.fract() == 0.0 && number.is_finite() && number.abs() <= Self::MAX_SAFE_INTEGER
        } else {
            false
        }
    }

    #[prop("parseFloat")]
    #[must_use]
    pub fn parse_float(string: &str) -> f64 {
        parse_float(string)
    }

    #[prop("parseInt")]
    #[must_use]
    pub fn parse_int(string: &str, radix: Option<u32>) -> f64 {
        parse_int(string, radix)
    }
}

impl Constructor for NumberConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Res<ObjectHandle> {
        let str = match args.first() {
            Some(v) => Self::construct_from(realm, v)?,
            None => 0.0,
        };

        let obj = NumberObj::with_number(realm, str)?;

        Ok(obj)
    }
}

impl Func for NumberConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let str = match args.first() {
            Some(v) => Self::construct_from(realm, v)?,
            None => 0.0,
        };

        Ok(str.into())
    }
}

impl NumberObj {
    #[allow(clippy::new_ret_no_self, dead_code)]
    pub fn new(realm: &Realm) -> crate::Res<ObjectHandle> {
        Self::with_number(realm, 0.0)
    }

    pub fn with_number(realm: &Realm, number: impl Into<f64>) -> crate::Res<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableNumberObj {
                object: MutObject::with_proto(realm.intrinsics.number.clone()),
                number: number.into(),
            }),
        };

        Ok(this.into_object())
    }
}

#[properties_new(default_null(number), constructor(NumberConstructor::new))]
impl NumberObj {
    #[prop("toString")]
    fn to_string(&self, radix: Option<u32>) -> Res<YSString> {
        let inner = self.inner.try_borrow()?;

        let num = inner.number;

        if num.is_nan() {
            check_radix_opt(radix)?;

            return Ok("NaN".into());
        }

        if num.is_infinite() {
            check_radix_opt(radix)?;

            return Ok(if num.is_sign_positive() {
                "Infinity".into()
            } else {
                "-Infinity".into()
            });
        }

        radix.map_or_else(
            || Ok(fmt_num(num)),
            |radix| float_to_string_with_radix(num, radix),
        )
    }

    #[prop("toExponential")]
    fn to_exponential(&self, fraction_digits: Option<u32>) -> Res<String> {
        let inner = self.inner.try_borrow()?;

        let num = inner.number;

        if num.is_nan() {
            return Ok("NaN".to_owned());
        }

        if num.is_infinite() {
            return Ok(if num.is_sign_positive() {
                "Infinity".to_owned()
            } else {
                "-Infinity".to_owned()
            });
        }

        let fraction_digits = fraction_digits.unwrap_or(0);

        let result = format!("{:.1$e}", num, fraction_digits as usize);

        let result = result.replace('e', "e+");

        Ok(result)
    }

    #[prop("toFixed")]
    fn to_fixed(&self, fraction_digits: Option<u32>) -> Res<String> {
        let inner = self.inner.try_borrow()?;

        let num = inner.number;

        if num.is_nan() {
            return Ok("NaN".to_owned());
        }

        if num.is_infinite() {
            return Ok(if num.is_sign_positive() {
                "Infinity".to_owned()
            } else {
                "-Infinity".to_owned()
            });
        }

        let fraction_digits = fraction_digits.unwrap_or(0);
        let result = format!("{:.1$}", num, fraction_digits as usize);

        Ok(result)
    }

    #[prop("toPrecision")]
    fn to_precision(&self, precision: Option<u32>) -> Res<String> {
        const MAX_PRECISION: u32 = 2000;

        let inner = self.inner.try_borrow()?;

        let num = inner.number;

        if num.is_nan() {
            return Ok("NaN".to_owned());
        }

        if num.is_infinite() {
            return Ok(if num.is_sign_positive() {
                "Infinity".to_owned()
            } else {
                "-Infinity".to_owned()
            });
        }

        let Some(precision) = precision else {
            return Ok(num.to_string());
        };

        if num > 10f64.powi(precision as i32) {
            let result = format!("{:.1$e}", num, precision.saturating_sub(1) as usize);

            let result = result.replace('e', "e+");

            return Ok(result);
        }

        let num_digits = num.log10().ceil();

        let num_digits = if num_digits.is_infinite() || num_digits.is_nan() {
            1
        } else {
            num_digits as i32
        };

        let precision = if num_digits.is_negative() {
            precision + num_digits.unsigned_abs()
        } else {
            precision.saturating_sub(num_digits as u32)
        };

        if precision > MAX_PRECISION {
            return Err(crate::Error::range(
                "toPrecision() precision argument must be between 0 and 2000",
            ));
        }

        let result = format!("{:.1$}", num, precision as usize);

        Ok(result)
    }

    #[prop("valueOf")]
    fn value_of(&self) -> f64 {
        let inner = self.inner.borrow();

        inner.number
    }
}

pub fn check_radix_opt(radix: Option<u32>) -> Res {
    radix.map_or_else(|| Ok(()), check_radix)
}

pub fn check_radix(radix: u32) -> Res {
    if (2..=36).contains(&radix) {
        Ok(())
    } else {
        Err(crate::Error::range(
            "toString() radix argument must be between 2 and 36",
        ))
    }
}

//TODO: find a better way to do this
fn float_to_string_with_radix(value: f64, radix: u32) -> crate::Res<YSString> {
    if radix == 10 {
        return Ok(fmt_num(value));
    }

    const PRECISION: usize = 5;

    check_radix(radix)?;

    let is_negative = value < 0.0;
    let value = value.abs();

    let int_part = value.trunc() as u64;
    let int_str = num_bigint::BigUint::from(int_part).to_str_radix(radix);

    let mut result = String::new();
    if is_negative {
        result.push('-');
    }
    result.push_str(&int_str);

    let mut frac_part = value.fract();
    if frac_part > 0.0 && PRECISION > 0 {
        result.push('.');

        for _ in 0..PRECISION {
            frac_part *= f64::from(radix);
            let digit = frac_part.trunc() as u32;

            if digit < 10 {
                result.push(
                    char::from_digit(digit, radix)
                        .ok_or(crate::Error::new("Failed to convert digit to char"))?,
                );
            } else {
                result.push((b'a' + (digit - 10) as u8) as char);
            }

            frac_part -= f64::from(digit);
            if frac_part == 0.0 {
                break;
            }
        }
    }

    Ok(result.into())
}

fn parse_float(string: &str) -> f64 {
    let string = string.trim();

    if string.is_empty() {
        return f64::NAN;
    }

    if string.starts_with("Infinity") {
        return f64::INFINITY;
    }

    if string.starts_with("-Infinity") {
        return f64::NEG_INFINITY;
    }

    let mut idx = 0;
    let mut had_dot = false;
    let mut had_e = false;
    let chars: Vec<char> = string.chars().collect();

    while idx < chars.len() {
        let c = chars[idx];

        if c.is_ascii_digit() {
            idx += 1;
        } else if c == '.' {
            if had_dot || had_e {
                break;
            }
            had_dot = true;
            idx += 1;
        } else if c == 'e' || c == 'E' {
            if had_e {
                break;
            }
            had_e = true;
            idx += 1;

            if idx < chars.len() && (chars[idx] == '+' || chars[idx] == '-') {
                idx += 1;
            }
        } else if (c == '+' || c == '-') && idx == 0 {
            idx += 1;
        } else {
            break;
        }
    }

    if idx > 0 {
        if let Ok(x) = string[..idx].parse::<f64>() {
            return x;
        }
    }

    f64::NAN
}

fn parse_int(string: &str, radix: Option<u32>) -> f64 {
    let radix = radix.unwrap_or(10);

    let radix = if (2..=36).contains(&radix) { radix } else { 10 };

    let string = string.trim();

    if string.is_empty() {
        return f64::NAN;
    }

    let mut idx = 0;

    for c in string.chars() {
        if c.is_numeric() || c == '-' || c == '+' {
            idx += c.len_utf8();
        } else {
            if idx > 0 {
                let Some(Ok(x)) = string.get(..idx).map(|s| i32::from_str_radix(s, radix)) else {
                    return f64::NAN;
                };

                return f64::from(x);
            }

            break;
        }
    }

    if idx > 0 {
        let Some(Ok(x)) = string.get(..idx).map(|s| i32::from_str_radix(s, radix)) else {
            return f64::NAN;
        };

        return f64::from(x);
    }

    f64::NAN
}

#[must_use]
pub fn get_is_nan(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "isNaN",
        |args, _, realm| {
            Ok(Value::Boolean(if let Some(val) = args.first() {
                val.to_number(realm)?.is_nan()
            } else {
                true
            }))
        },
        realm,
        1,
    )
}

#[must_use]
pub fn get_is_finite(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "isFinite",
        |args, _, realm| {
            Ok(Value::Boolean(if let Some(val) = args.first() {
                val.to_number(realm)?.is_finite()
            } else {
                false
            }))
        },
        realm,
        1,
    )
}

#[must_use]
pub fn get_parse_int(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "parseInt",
        |args, _, realm| {
            let radix = args
                .get(1)
                .and_then(|v| v.to_number(realm).ok())
                .map(|v| v as u32);

            let str = args
                .first()
                .and_then(|v| v.to_string(realm).ok())
                .unwrap_or_default();

            Ok(Value::Number(NumberConstructor::parse_int(&str, radix)))
        },
        realm,
        2,
    )
}

#[must_use]
pub fn get_parse_float(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "parseFloat",
        |args, _, realm| {
            let str = args
                .first()
                .and_then(|v| v.to_string(realm).ok())
                .unwrap_or_default();

            Ok(Value::Number(NumberConstructor::parse_float(&str)))
        },
        realm,
        1,
    )
}
