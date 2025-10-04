use crate::{MutObject, ObjectHandle, Realm};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use crate::value::Obj;

#[object]
#[derive(Debug)]
pub struct Timer {}

impl Timer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm) -> crate::Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableTimer {
                object: MutObject::new(realm),
            }),
        };

        this.initialize(realm.intrinsics.func.clone().into(), realm)?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl Timer {
    pub fn wait(ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}
