use std::cell::RefCell;
use std::fs;
use yavashark_macro::{object, properties_new};
use yavashark_value::Obj;
use crate::{Error, NativeFunction, Realm, ValueResult, Result, ObjectHandle, MutObject};




#[object]
#[derive(Debug)]
pub struct Fs {}


impl Fs {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &Realm) -> Result<ObjectHandle> {
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
    fn open(path: String) -> Result<String> {
        let contents = fs::read_to_string(&path)
            .map_err(|e| Error::new_error(e.to_string()))?;
        
        Ok(contents)
    }
}