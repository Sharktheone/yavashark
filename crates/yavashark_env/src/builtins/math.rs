use crate::partial_init::Initializer;
use crate::utils::ValueIterator;
use crate::value::Obj;
use crate::{Error, MutObject, ObjectHandle, Realm, Res, Symbol, Value};
use num_traits::{One, Zero};
use std::cell::RefCell;
use xsum::Xsum;
use yavashark_macro::{object, properties_new};

#[object]
#[derive(Debug)]
pub struct Math {}

impl Math {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableMath {
                object: MutObject::with_proto(realm.intrinsics.obj.clone()),
            }),
        };

        this.initialize(realm)?;

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

    #[prop(Symbol::TO_STRING_TAG)]
    #[configurable]
    const TO_STRING_TAG: &'static str = "Math";

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
        if value.is_infinite() {
            return 32.0;
        }

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
        half::f16::from_f64(value).to_f64()
    }

    const fn fround(value: f64) -> f64 {
        value as f32 as f64
    }

    #[length(2)]
    fn hypot(vals: &[f64]) -> f64 {
        let mut res = 0.0f64;

        for &v in vals {
            res = res.hypot(v);
        }

        res
    }


    const fn imul(left: u32, right: u32) -> i32 {
        left.wrapping_mul(right) as i32
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

    #[length(2)]
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

    #[length(2)]
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
        if exponent.is_zero() {
            return 1.0;
        }

        if base.is_nan() || exponent.is_nan() {
            return f64::NAN;
        }

        if base.abs().is_one() && exponent.is_infinite() {
            return f64::NAN;
        }

        base.powf(exponent)
    }

    fn random() -> f64 {
        rand::random()
    }

    fn round(value: f64) -> f64 {
        if (value.fract() + 0.5).abs() < f64::EPSILON {
            return value.ceil();
        }

        value.round()
    }

    const fn sign(value: f64) -> f64 {
        if value == -0.0 && value.is_sign_negative() {
            return -0.0;
        }

        if value == 0.0 {
            return 0.0;
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
    fn sum_precise(iter: &ObjectHandle, #[realm] realm: &mut Realm) -> Res<f64> {
        if !iter.contains_key(Symbol::ITERATOR.into(), realm)? {
            return Err(Error::ty(
                "Value is not iterable: missing @@iterator method",
            ));
        }


        let iter = ValueIterator::new_obj(iter, realm)?;
        let mut sum = xsum::XsumAuto::new();

        while let Some(value) = iter.next(realm)? {
            if let Value::Number(num) = value {
                sum.add(num);
            } else {
                iter.close(realm)?;
                return Err(Error::ty("Iterator value is not a number"));
            }
        }

        iter.close(realm)?;

        Ok(sum.sum())
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

impl Initializer<ObjectHandle> for Math {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle> {
        Math::new(realm)
    }
}
