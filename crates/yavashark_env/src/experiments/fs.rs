use crate::{Error, MutObject, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use std::fs;
use yavashark_macro::{object, properties_new};
use yavashark_value::Obj;

#[object]
#[derive(Debug)]
pub struct Fs {}

impl Fs {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm) -> Res<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableFs {
                object: MutObject::new(realm),
            }),
        };

        this.initialize(realm.intrinsics.func.clone().into())?;

        Ok(this.into_object())
    }
}

#[properties_new(raw)]
impl Fs {
    fn open(path: &str) -> Res<String> {
        let contents = fs::read_to_string(path).map_err(|e| Error::new_error(e.to_string()))?;

        Ok(contents)
    }
}
