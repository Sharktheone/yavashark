use crate::utils::ValueIterator;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Math {}

impl Math {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: ObjectHandle, func: ObjectHandle) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableMath {
                object: MutObject::with_proto(proto.into()),
            }),
        };

        this.initialize(func.into())?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl Math {
    const E: f64 = std::f64::consts::E;
    const LN10: f64 = std::f64::consts::LN_10;
    const LN2: f64 = std::f64::consts::LN_2;
    const LOG10E: f64 = std::f64::consts::LOG10_E;
    const LOG2E: f64 = std::f64::consts::LOG2_E;
    const PI: f64 = std::f64::consts::PI;
    const SQRT1_2: f64 = std::f64::consts::FRAC_1_SQRT_2;
    const SQRT2: f64 = std::f64::consts::SQRT_2;

    const fn abs(value: f64) -> f64 {
        value.abs()
    }

    fn acos(value: f64) -> f64 {
        value.acos()
    }

    fn acosh(value: f64) -> f64 {
        value.acosh()
    }

    fn asin(value: f64) -> f64 {
        value.asin()
    }

    fn asinh(value: f64) -> f64 {
        value.asinh()
    }

    fn atan(value: f64) -> f64 {
        value.atan()
    }

    fn atan2(left: f64, right: f64) -> f64 {
        left.atan2(right)
    }

    fn atanh(value: f64) -> f64 {
        value.atanh()
    }

    fn cbrt(value: f64) -> f64 {
        value.cbrt()
    }

    fn ceil(value: f64) -> f64 {
        value.ceil()
    }

    const fn clz32(value: f64) -> f64 {
        (value as i64 as u32).leading_zeros() as f64
    }

    fn cos(value: f64) -> f64 {
        value.cos()
    }

    fn cosh(value: f64) -> f64 {
        value.cosh()
    }

    fn exp(value: f64) -> f64 {
        value.exp()
    }

    fn expm1(value: f64) -> f64 {
        value.exp_m1()
    }

    fn floor(value: f64) -> f64 {
        value.floor()
    }

    fn f16round(value: f64) -> f64 {
        value.round() //TODO: Implement f16round
    }

    fn fround(value: f64) -> f64 {
        value.round()
    }

    fn hypot(left: f64, right: f64) -> f64 {
        left.hypot(right)
    }

    const fn imul(left: f64, right: f64) -> f64 {
        (left as i32 * right as i32) as f64
    }

    fn log(value: f64) -> f64 {
        value.ln()
    }

    fn log10(value: f64) -> f64 {
        value.log10()
    }

    fn log1p(value: f64) -> f64 {
        value.ln_1p()
    }

    fn log2(value: f64) -> f64 {
        value.log2()
    }

    fn max(#[variadic] args: &[Value], #[realm] realm: &mut Realm) -> Res<f64> {
        Ok(args
            .iter()
            .try_fold(None::<f64>, |acc, v| {
                let val = v.to_number(realm)?;

                if val.is_nan() {
                    return Ok::<Option<f64>, Error>(Some(f64::NAN));
                }

                Ok::<Option<f64>, Error>(Some(acc.map_or(val, |acc| {
                    if acc.is_nan() {
                        return f64::NAN;
                    }

                    float_max(acc, val)
                })))
            })?
            .unwrap_or(f64::NEG_INFINITY))
    }

    fn min(#[variadic] args: &[Value], #[realm] realm: &mut Realm) -> Res<f64> {
        Ok(args
            .iter()
            .try_fold(None::<f64>, |acc, v| {
                let val = v.to_number(realm)?;

                if val.is_nan() {
                    return Ok::<Option<f64>, Error>(Some(f64::NAN));
                }

                Ok::<Option<f64>, Error>(Some(acc.map_or(val, |acc| {
                    if acc.is_nan() {
                        return f64::NAN;
                    }
                    float_min(acc, val)
                })))
            })?
            .unwrap_or(f64::INFINITY))
    }

    fn pow(base: f64, exponent: f64) -> f64 {
        if base.is_nan() || exponent.is_nan() {
            return f64::NAN;
        }
        
        if exponent.is_infinite() {
            return f64::NAN;
        }
        
        base.powf(exponent)
    }

    fn random() -> f64 {
        rand::random()
    }

    fn round(value: f64) -> f64 {
        value.round()
    }

    const fn sign(value: f64) -> f64 {
        if value == -0.0 && value.is_sign_negative() {
            return -0.0;
        }

        value.signum()
    }

    fn sin(value: f64) -> f64 {
        value.sin()
    }

    fn sinh(value: f64) -> f64 {
        value.sinh()
    }

    fn sqrt(value: f64) -> f64 {
        value.sqrt()
    }

    #[prop("sumPrecise")]
    fn sum_precise(iter: &Value, #[realm] realm: &mut Realm) -> Res<f64> {
        let mut sum = 0.0;

        let iter = ValueIterator::new(iter, realm)?;

        while let Some(value) = iter.next(realm)? {
            sum += value.to_number(realm).unwrap_or(0.0);
        }

        Ok(sum)
    }

    fn tan(value: f64) -> f64 {
        value.tan()
    }

    fn tanh(value: f64) -> f64 {
        value.tanh()
    }

    fn trunc(value: f64) -> f64 {
        value.trunc()
    }
}

fn float_max(left: f64, right: f64) -> f64 {
    #[allow(clippy::float_cmp)]
    if left > right {
        left
    } else if right > left {
        right
    } else if left == right {
        if left.is_sign_positive() && right.is_sign_negative() {
            left
        } else {
            right
        }
    } else {
        left + right
    }
}

fn float_min(left: f64, right: f64) -> f64 {
    #[allow(clippy::float_cmp)]
    if left < right {
        left
    } else if right < left {
        right
    } else if left == right {
        if left.is_sign_negative() && right.is_sign_positive() {
            left
        } else {
            right
        }
    } else {
        left + right
    }
}
