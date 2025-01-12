use crate::realm::Realm;
use crate::{Error, MutObject, NativeConstructor, ObjectHandle, Result, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties};

#[must_use]
pub fn get_error(realm: &Realm) -> Value {
    NativeConstructor::special(
        "error".to_string(),
        || {
            // let message = args
            //     .first()
            //     .map_or(String::new(), std::string::ToString::to_string);
            //
            // let err = ErrorObj::raw_from(message, realm);
            //
            // this.exchange(Box::new(err))?;
            //
            // Ok(Value::Undefined)

            todo!()
        },
        Some(Box::new(|realm, _| {
            let err: Value = ErrorObj::new(Error::new("error not initialized"), realm).into();

            err
        })),
        realm,
    )
    .into()
}

#[object(to_string)]
#[derive(Debug)]
#[allow(dead_code)]
pub struct ErrorObj {
    pub(crate) error: Error,
}

impl ErrorObj {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(error: Error, realm: &Realm) -> ObjectHandle {
        let this = Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.error.clone().into()),
            }),
            error,
        };

        ObjectHandle::new(this)
    }

    #[must_use]
    pub fn new_from(message: String, realm: &Realm) -> ObjectHandle {
        let this = Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.error.clone().into()),
            }),
            error: Error::unknown_error(message),
        };

        ObjectHandle::new(this)
    }

    #[must_use]
    pub fn raw_from(message: String, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.error.clone().into()),
            }),
            error: Error::unknown_error(message),
        }
    }

    pub fn override_to_string(&self, _: &mut Realm) -> Result<String> {
        Ok(self.error.to_string())
    }

    pub fn override_to_string_internal(&self) -> Result<String> {
        Ok(self.error.to_string())
    }
}

#[properties]
impl ErrorObj {
    #[get(message)]
    pub fn get_message(&self, _: Vec<Value>, realm: &mut Realm) -> ValueResult {
        Ok(self.error.message(realm)?.into())
    }
}
