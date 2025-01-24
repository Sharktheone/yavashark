use crate::{MutObject, Object, ObjectHandle, Realm, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, Func, Obj};

#[object]
#[derive(Debug)]
pub struct NumberObj {
    #[mutable]
    #[primitive]
    number: f64,
}

#[object(constructor, function)]
#[derive(Debug)]
pub struct NumberConstructor {}

impl NumberConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableNumberConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.initialize(func.copy())?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl NumberConstructor {
    const EPSILON: f64 = f64::EPSILON;
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    const MIN_SAFE_INTEGER: f64 = -9_007_199_254_740_991.0;

    const MAX_VALUE: f64 = f64::MAX;
    const MIN_VALUE: f64 = f64::MIN;

    const NEGATIVE_INFINITY: f64 = f64::NEG_INFINITY;
    const POSITIVE_INFINITY: f64 = f64::INFINITY;
    #[prop("NaN")]
    const NAN: f64 = f64::NAN;

    #[prop("isFinite")]
    pub const fn is_finite(number: f64) -> bool {
        number.is_finite()
    }

    #[prop("isNaN")]
    pub const fn is_nan(number: f64) -> bool {
        number.is_nan()
    }

    #[prop("isInteger")]
    pub fn is_integer(number: f64) -> bool {
        number.fract() == 0.0
    }

    #[prop("isSafeInteger")]
    pub fn is_safe_integer(number: f64) -> bool {
        number.is_finite() && number.abs() <= Self::MAX_SAFE_INTEGER
    }

    #[prop("parseFloat")]
    pub fn parse_float(string: String) -> f64 {
        string.parse().unwrap_or(f64::NAN)
    }

    #[prop("parseInt")]
    pub fn parse_int(string: String, radix: u32) -> f64 {
        i64::from_str_radix(&string, radix)
            .map(|n| n as f64)
            .unwrap_or(f64::NAN)
    }
}

impl Constructor<Realm> for NumberConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let str = match args.first() {
            Some(v) => v.to_number(realm)?,
            None => 0.0,
        };

        let obj = NumberObj::with_string(realm, str)?;

        Ok(obj.into())
    }
}

impl Func<Realm> for NumberConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let str = match args.first() {
            Some(v) => v.to_number(realm)?,
            None => 0.0,
        };

        Ok(str.into())
    }
}

impl NumberObj {
    #[allow(clippy::new_ret_no_self, dead_code)]
    pub fn new(realm: &Realm) -> crate::Result<ObjectHandle> {
        Self::with_string(realm, 0.0)
    }

    pub fn with_string(realm: &Realm, number: impl Into<f64>) -> crate::Result<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableNumberObj {
                object: MutObject::with_proto(realm.intrinsics.string.clone().into()),
                number: number.into(),
            }),
        };

        Ok(this.into_object())
    }
}

#[properties_new(constructor(NumberConstructor::new))]
impl NumberObj {}
