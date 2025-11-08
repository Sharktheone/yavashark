use std::cell::RefCell;
use crate::builtins::check_radix;
use crate::conversion::downcast_obj;
use crate::value::{Obj};
use crate::{MutObject, ObjectHandle, Realm, Res, Value, ValueResult};
use num_bigint::BigInt;
use std::rc::Rc;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct BigIntObj {
    #[mutable]
    #[primitive]
    big_int: Rc<BigInt>,
}


impl BigIntObj {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm, big_int: Rc<BigInt>) -> Res<ObjectHandle> {
        Ok(Self {
            inner: RefCell::new(MutableBigIntObj {
                object: MutObject::with_proto(
                    realm.intrinsics.clone_public().bigint.get(realm)?.clone(),
                ),
                big_int,
            }),
        }
            .into_object())
    }
}

#[props(intrinsic_name = bigint)]
impl BigIntObj {
    #[call_constructor]
    fn call(realm: &mut Realm, bint: Value) -> ValueResult {
        Ok(bint.to_big_int(realm)?.into())
    }

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