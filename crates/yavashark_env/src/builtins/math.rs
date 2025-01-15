use crate::{MutObject, ObjectHandle, Realm, Result, Value};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Math {}

impl Math {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(proto: ObjectHandle, func: ObjectHandle) -> Result<ObjectHandle> {
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

    fn abs(value: f64) -> f64 {
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
        value.to_bits().leading_zeros() as f64
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

    fn max(#[variadic] args: &[Value], #[realm] realm: &mut Realm) -> Result<f64> {
        args.iter()
            .try_fold(f64::NEG_INFINITY, |acc, v| Ok(acc.max(v.to_number(realm)?)))
    }

    fn min(#[variadic] args: &[Value], #[realm] realm: &mut Realm) -> Result<f64> {
        args.iter()
            .try_fold(f64::INFINITY, |acc, v| Ok(acc.min(v.to_number(realm)?)))
    }

    fn pow(base: f64, exponent: f64) -> f64 {
        base.powf(exponent)
    }

    fn random() -> f64 {
        rand::random()
    }

    fn round(value: f64) -> f64 {
        value.round()
    }

    fn sign(value: f64) -> f64 {
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
