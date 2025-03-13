use std::cell::RefCell;
use crate::{MutObject, ObjectHandle, Realm};
use num_bigint::BigInt;
use yavashark_macro::{object, props};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Instant {
    #[mutable]
    stamp: BigInt,
}


impl Instant {
    #[must_use]
    pub fn new(stamp: BigInt, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableInstant {
                object: MutObject::with_proto(realm.intrinsics.temporal_instant.clone().into()),
                stamp,
            }),
        }
    }
}

#[props]
impl Instant {
    #[constructor]
    fn construct(epoch: BigInt, #[realm] realm: &Realm) -> ObjectHandle {
        Self::new(epoch, realm).into_object()
    }
}
