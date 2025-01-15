use crate::{MutObject, ObjectHandle, Realm};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Tcp {}

impl Tcp {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm) -> crate::Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableTcp {
                object: MutObject::new(realm),
            }),
        };

        this.initialize(realm.intrinsics.func.clone().into())?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl Tcp {}
