use crate::realm::Realm;
use crate::{Error, MutObject, NativeConstructor, ObjectHandle, Result, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties};

#[must_use]
pub fn get_error(realm: &Realm) -> Value {
    NativeConstructor::special(
        "error".to_string(),
        |args, realm| {
            let message = args
                .first()
                .map_or(String::new(), std::string::ToString::to_string);

            let obj: Value = ErrorObj::new(Error::unknown_error(message), realm).into();

            Ok(obj)
        },
        realm,
    )
    .into()
}

#[object(to_string)]
#[derive(Debug)]
#[allow(dead_code)]
pub struct ErrorObj {
    #[mutable]
    pub(crate) error: Error,
}

impl ErrorObj {
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(error: Error, realm: &Realm) -> ObjectHandle {
        let this = Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.error.clone().into()),
                error,
            }),
        };

        ObjectHandle::new(this)
    }

    #[must_use]
    pub fn new_from(message: String, realm: &Realm) -> ObjectHandle {
        let this = Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.error.clone().into()),
                error: Error::unknown_error(message),
            }),
        };

        ObjectHandle::new(this)
    }

    #[must_use]
    pub fn raw(error: Error, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.error.clone().into()),
                error,
            }),
        }
    }

    #[must_use]
    pub fn raw_from(message: String, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.error.clone().into()),
                error: Error::unknown_error(message),
            }),
        }
    }

    pub fn override_to_string(&self, _: &mut Realm) -> Result<String> {
        let inner = self.inner.try_borrow()?;
        Ok(inner.error.to_string())
    }

    pub fn override_to_string_internal(&self) -> Result<String> {
        let inner = self.inner.try_borrow()?;
        Ok(inner.error.to_string())
    }
}

#[properties]
impl ErrorObj {
    #[get(message)]
    pub fn get_message(&self, _: Vec<Value>, realm: &mut Realm) -> ValueResult {
        let inner = self.inner.try_borrow()?;
        Ok(inner.error.message(realm)?.into())
    }
}
