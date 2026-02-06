use crate::partial_init::Initializer;
use crate::utils::ProtoDefault;
use crate::value::{fmt_num, Constructor, Func, Obj};
use crate::{MutObject, NativeFunction, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use num_bigint::Sign;
use num_traits::ToPrimitive;
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_string::YSString;

/// ToIntegerOrInfinity operation
pub fn to_integer_or_infinity(n: f64) -> f64 {
    if n.is_nan() {
        0.0
    } else if n == 0.0 || n.is_infinite() {
        n
    } else {
        n.trunc()
    }
}

fn format_exponential(x: f64, f: usize) -> YSString {
    if x == 0.0 {
        let mut s = String::from("0");
        if f > 0 {
            s.push('.');
            s.push_str(&"0".repeat(f));
        }
        s.push_str("e+0");
        return s.into();
    }

    let is_negative = x < 0.0;
    let x = x.abs();

    let e = x.log10().floor() as i32;

    if f > 15 {
        let formatted = format!("{:.prec$e}", if is_negative { -x } else { x }, prec = f);
        let parts: Vec<&str> = formatted.split('e').collect();
        let mantissa = parts[0];
        let exp: i32 = parts.get(1).and_then(|ee| ee.parse().ok()).unwrap_or(0);

        let mut s = String::new();
        s.push_str(mantissa);
        s.push('e');
        if exp >= 0 {
            s.push('+');
        }
        s.push_str(&exp.to_string());
        return s.into();
    }

    let scale = 10f64.powi(f as i32 - e);
    let n = (x * scale).round() as i64;

    let max_n = 10i64.pow((f + 1) as u32);
    let (n, e) = if n >= max_n {
        (n / 10, e + 1)
    } else if n < 10i64.pow(f as u32) && f > 0 {
        (n * 10, e - 1)
    } else {
        (n, e)
    };

    let digits = format!("{:0>width$}", n, width = f + 1);

    let mut s = String::new();
    if is_negative {
        s.push('-');
    }

    if f > 0 {
        let (first, rest) = digits.split_at(1);
        s.push_str(first);
        s.push('.');
        s.push_str(rest);
    } else {
        s.push_str(&digits);
    }

    s.push('e');
    if e >= 0 {
        s.push('+');
    }
    s.push_str(&e.to_string());

    s.into()
}

#[object]
#[derive(Debug)]
pub struct NumberObj {
    #[mutable]
    #[primitive]
    number: f64,
}

impl ProtoDefault for NumberObj {
    fn proto_default(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableNumberObj {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().number.get(realm)?.clone(),
                ),
                number: 0.0,
            }),
        })
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
        use crate::value::{Obj, Variable};

        let mut this = Self {
            inner: RefCell::new(MutableNumberConstructor {
                object: MutObject::with_proto(func.clone()),
            }),
        };

        this.initialize(realm)?;

        let handle = this.into_object();

        let parse_float = realm
            .intrinsics
            .clone_public()
            .parse_float
            .get(realm)?
            .clone();
        let parse_int = realm
            .intrinsics
            .clone_public()
            .parse_int
            .get(realm)?
            .clone();

        handle.define_property_attributes(
            "parseFloat".into(),
            Variable::write_config(parse_float.into()),
            realm,
        )?;
        handle.define_property_attributes(
            "parseInt".into(),
            Variable::write_config(parse_int.into()),
            realm,
        )?;

        Ok(handle)
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

impl NumberConstructor {
    pub const MAX_SAFE_INTEGER_U: u64 = 9_007_199_254_740_991;
    pub const MIN_SAFE_INTEGER_U: i64 = -9_007_199_254_740_991;
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
    pub fn new(realm: &mut Realm) -> crate::Res<ObjectHandle> {
        Self::with_number(realm, 0.0)
    }

    pub fn with_number(realm: &mut Realm, number: impl Into<f64>) -> crate::Res<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableNumberObj {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().number.get(realm)?.clone(),
                ),
                number: number.into(),
            }),
        };

        Ok(this.into_object())
    }
}

#[properties_new(
    intrinsic_name(number),
    default_null(number),
    constructor(NumberConstructor::new),
    constructor_length = 1,
    constructor_name(Number)
)]
impl NumberObj {
    #[prop("toString")]
    #[length(1)]
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
    #[length(1)]
    fn to_exponential(&self, fraction_digits: Value, #[realm] realm: &mut Realm) -> Res<YSString> {
        let inner = self.inner.try_borrow()?;
        let num = inner.number;

        let fraction_digits_undefined = fraction_digits.is_undefined();
        let f = if fraction_digits_undefined {
            0
        } else {
            to_integer_or_infinity(fraction_digits.to_number(realm)?) as isize
        };

        if !num.is_finite() {
            if num.is_nan() {
                return Ok("NaN".into());
            }
            return Ok(if num.is_sign_positive() {
                "Infinity".into()
            } else {
                "-Infinity".into()
            });
        }

        if !(0..=100).contains(&f) {
            return Err(crate::Error::range(
                "toExponential() argument must be between 0 and 100",
            ));
        }
        let f = f as usize;

        if fraction_digits_undefined {
            let x = num;
            let mut s = String::new();
            let x = if x < 0.0 {
                s.push('-');
                -x
            } else {
                x
            };

            if x == 0.0 {
                s.push_str("0e+0");
                return Ok(s.into());
            }

            let formatted = format!("{x:e}");
            let parts: Vec<&str> = formatted.split('e').collect();
            let mantissa = parts[0];
            let exp: i32 = parts.get(1).and_then(|e| e.parse().ok()).unwrap_or(0);

            let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');

            s.push_str(mantissa);
            s.push('e');
            if exp >= 0 {
                s.push('+');
            }
            s.push_str(&exp.to_string());
            return Ok(s.into());
        }

        Ok(format_exponential(num, f))
    }

    #[prop("toFixed")]
    #[length(1)]
    fn to_fixed(&self, fraction_digits: Value, #[realm] realm: &mut Realm) -> Res<YSString> {
        let inner = self.inner.try_borrow()?;
        let num = inner.number;

        let f = to_integer_or_infinity(fraction_digits.to_number(realm)?);

        if !f.is_finite() {
            return Err(crate::Error::range(
                "toFixed() digits argument must be between 0 and 100",
            ));
        }

        if f < 0.0 || f > 100.0 {
            return Err(crate::Error::range(
                "toFixed() digits argument must be between 0 and 100",
            ));
        }
        let f = f as usize;

        if !num.is_finite() {
            if num.is_nan() {
                return Ok("NaN".into());
            }
            return Ok(if num.is_sign_positive() {
                "Infinity".into()
            } else {
                "-Infinity".into()
            });
        }

        if num.abs() >= 1e21 {
            return Ok(fmt_num(num));
        }

        let result = format!("{:.1$}", num, f);
        Ok(result.into())
    }

    #[prop("toPrecision")]
    #[length(1)]
    fn to_precision(&self, precision: Value, #[realm] realm: &mut Realm) -> Res<YSString> {
        let inner = self.inner.try_borrow()?;
        let num = inner.number;

        if precision.is_undefined() {
            return Ok(fmt_num(num));
        }

        let p = to_integer_or_infinity(precision.to_number(realm)?);

        if !num.is_finite() {
            if num.is_nan() {
                return Ok("NaN".into());
            }
            return Ok(if num.is_sign_positive() {
                "Infinity".into()
            } else {
                "-Infinity".into()
            });
        }

        if p < 1.0 || p > 100.0 {
            return Err(crate::Error::range(
                "toPrecision() argument must be between 1 and 100",
            ));
        }
        let p = p as usize;

        let x = num;
        let mut s = String::new();
        let x = if x < 0.0 {
            s.push('-');
            -x
        } else {
            x
        };

        if x == 0.0 {
            s.push_str(&"0".repeat(p));
            if p > 1 {
                s.insert(1, '.');
            }
            return Ok(s.into());
        }

        let e = x.log10().floor() as i32;

        if e < -6 || e >= p as i32 {
            let formatted = format!("{:.1$e}", x, p - 1);
            let parts: Vec<&str> = formatted.split('e').collect();
            let mantissa = parts[0];
            let exp: i32 = parts.get(1).and_then(|ee| ee.parse().ok()).unwrap_or(0);

            let (c, e_abs) = if exp >= 0 { ("+", exp) } else { ("-", -exp) };

            s.push_str(mantissa);
            s.push('e');
            s.push_str(c);
            s.push_str(&e_abs.to_string());
            return Ok(s.into());
        }

        let decimal_places = if e >= 0 {
            (p as i32 - e - 1).max(0) as usize
        } else {
            ((-e - 1) + p as i32) as usize
        };

        let formatted = format!("{:.1$}", x, decimal_places);
        s.push_str(&formatted);

        Ok(s.into())
    }

    #[prop("valueOf")]
    fn value_of(&self) -> f64 {
        let inner = self.inner.borrow();

        inner.number
    }

    #[prop("toLocaleString")]
    fn to_locale_string(&self) -> Res<YSString> {
        self.to_string(None)
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
    if radix == Some(1) {
        return f64::NAN;
    }

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

pub struct IsNan;

impl Initializer<ObjectHandle> for IsNan {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_is_nan(realm))
    }
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

#[derive(Debug)]
pub struct IsFinite;

impl Initializer<ObjectHandle> for IsFinite {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_is_finite(realm))
    }
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

            Ok(Value::Number(parse_int(&str.as_str_lossy(), radix)))
        },
        realm,
        2,
    )
}

pub struct ParseInt;

impl Initializer<ObjectHandle> for ParseInt {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_parse_int(realm))
    }
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

            Ok(Value::Number(parse_float(&str.as_str_lossy())))
        },
        realm,
        1,
    )
}

pub struct ParseFloat;

impl Initializer<ObjectHandle> for ParseFloat {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(get_parse_float(realm))
    }
}
