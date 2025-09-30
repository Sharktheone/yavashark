use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Object, ObjectHandle, Realm, Res, Value};

#[object]
#[derive(Debug)]
pub struct Atomics {}

impl Atomics {
    pub fn new(realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableAtomics {
                object: MutObject::with_proto(realm.intrinsics.atomics.clone()),
            }),
        }
    }
}

#[props]
#[allow(unused)]
impl Atomics {
    pub fn add(ta: &ObjectHandle, index: usize, value: i32) {

    }

    pub fn and(ta: &ObjectHandle, index: usize, value: i32) {

    }


    #[prop("compareExchange")]
    pub fn compare_exchange(ta: &ObjectHandle, index: usize, expected: i32, replacement: i32) {

    }

    pub fn exchange(ta: &ObjectHandle, index: usize, value: i32) {

    }

    #[prop("isLockFree")]
    pub fn is_lock_free(size: usize) -> bool {
        matches!(size, 1 | 2 | 4 | 8)
    }

    pub fn load(ta: &ObjectHandle, index: usize) -> i32 {
        0
    }

    pub fn notify(ta: &ObjectHandle, index: usize, count: Option<u32>) -> u32 {
        count.unwrap_or(0)
    }

    pub fn or(ta: &ObjectHandle, index: usize, value: i32) {

    }

    fn pause(hint: Option<Value>) -> Res<()> {
        let Value::Number(n) = hint.unwrap_or(Value::from(0)) else {
            return Err(crate::Error::ty("hint must be a number"));
        };

        if n < 0.0 || n.fract() != 0.0 {
            return Err(crate::Error::ty("hint must be a non-negative integer"));
        }

        let hint = n as usize;

        let hint = hint.max(2 << 8);

        for _ in 0..hint {
            std::hint::spin_loop();
        }

        Ok(())
    }

    pub fn store(ta: &ObjectHandle, index: usize, value: i32) {

    }

    pub fn sub(ta: &ObjectHandle, index: usize, value: i32) {

    }

    pub fn wait(ta: &ObjectHandle, index: usize, value: i32, timeout: Option<u32>) -> String {
        "ok".into()
    }

    #[prop("waitAsync")]
    pub fn wait_async(ta: &ObjectHandle, index: usize, value: i32, timeout: Option<u32>, realm: &mut Realm) -> Res<ObjectHandle> {
        let obj = Object::new(realm);

        obj.set("async", false, realm)?;
        obj.set("value", "ok", realm)?;


        Ok(obj)
    }

}