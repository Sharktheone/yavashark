use crate::builtins::check_radix;
use crate::conversion::FromValueOutput;
use crate::{MutObject, Object, ObjectHandle, Realm, Value, ValueResult};
use num_bigint::BigInt;
use std::cell::RefCell;
use std::rc::Rc;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Func, Obj};

#[object]
#[derive(Debug)]
pub struct BigIntObj {
    #[mutable]
    #[primitive]
    big_int: Rc<BigInt>,
}

#[object(function)]
#[derive(Debug)]
pub struct BigIntConstructor {}

impl BigIntConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableBigIntConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        this.initialize(func.copy())?;

        Ok(this.into_object())
    }
}

impl Func<Realm> for BigIntConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let first = args.first().unwrap_or(&Value::Undefined);

        Ok(first.to_big_int(realm)?.into())

        // if num.is_nan() || num.is_infinite() {
        //     return Err(Error::ty_error(format!(
        //         "Cannot convert {} to BigInt",
        //         first.to_string(realm)?
        //     )));
        // }

        // Ok(BigInt::from(num as u128).into())
    }
}

impl BigIntObj {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(realm: &Realm, big_int: Rc<BigInt>) -> ObjectHandle {
        Self {
            inner: RefCell::new(MutableBigIntObj {
                object: MutObject::with_proto(realm.intrinsics.bigint.clone().into()),
                big_int,
            }),
        }
        .into_object()
    }
}

#[properties_new(constructor(BigIntConstructor::new))]
impl BigIntObj {
    #[prop("toString")]
    fn to_string(&self, radix: Option<u32>) -> ValueResult {
        let inner = self.inner.try_borrow()?;

        Ok(if let Some(radix) = radix {
            check_radix(radix)?;

            inner.big_int.to_str_radix(radix)
        } else {
            inner.big_int.to_string()
        }
        .into())
    }

    #[prop("valueOf")]
    fn value_of(#[this] this: Value) -> ValueResult {
        if let Value::BigInt(bi) = this {
            return Ok(bi.into());
        }

        let this = <&Self as FromValueOutput>::from_value_out(this)?;

        let inner = this.inner.try_borrow()?;

        Ok(inner.big_int.clone().into())
    }
}

#[properties_new(raw)]
impl BigIntConstructor {
    #[prop("asIntN")]
    pub fn int_n(bits: u64, bigint: BigInt) -> ValueResult {
        let mut mask = BigInt::from(1) << bits;
        mask -= 1;
        //TODO: this handles the sign bit incorrectly

        Ok((bigint & mask).into())
    }

    #[prop("asUintN")]
    pub fn uint_n(bits: u64, bigint: BigInt) -> ValueResult {
        let mut mask = BigInt::from(1) << bits;
        mask -= 1;

        Ok((bigint & mask).into())
    }
}
