use crate::builtins::check_radix;
use crate::conversion::downcast_obj;
use crate::value::{Func, Obj};
use crate::{MutObject, Object, ObjectHandle, Realm, Value, ValueResult};
use num_bigint::BigInt;
use std::cell::RefCell;
use std::rc::Rc;
use yavashark_macro::{object, properties_new};

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
    pub fn new(_: &Object, func: ObjectHandle, realm: &mut Realm) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableBigIntConstructor {
                object: MutObject::with_proto(func.clone()),
            }),
        };

        this.initialize(realm)?;

        Ok(this.into_object())
    }
}

impl Func for BigIntConstructor {
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
                object: MutObject::with_proto(realm.intrinsics.bigint.clone()),
                big_int,
            }),
        }
        .into_object()
    }
}

#[properties_new(intrinsic_name(bigint), constructor(BigIntConstructor::new))]
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

        let this = downcast_obj::<Self>(this)?;

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
