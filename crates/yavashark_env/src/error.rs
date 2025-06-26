use crate::realm::Realm;
use crate::{Error, MutObject, ObjectHandle, Res, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, props};
use yavashark_string::{ToYSString, YSString};
use yavashark_value::{CustomName, ErrorKind};

#[object(to_string, name)]
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
        let proto = match &error.kind {
            ErrorKind::Type(_) => realm.intrinsics.type_error.clone(),
            ErrorKind::Reference(_) => realm.intrinsics.reference_error.clone(),
            ErrorKind::Range(_) => realm.intrinsics.range_error.clone(),
            ErrorKind::Syntax(_) => realm.intrinsics.syntax_error.clone(),
            ErrorKind::Eval(_) => realm.intrinsics.eval_error.clone(),
            ErrorKind::URI(_) => realm.intrinsics.uri_error.clone(),
            _ => realm.intrinsics.error.clone(),
        };

        let this = Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(proto.into()),
                error,
            }),
        };

        ObjectHandle::new(this)
    }

    #[must_use]
    pub fn error_to_value(err: Error, realm: &Realm) -> Value {
        match err.kind {
            ErrorKind::Throw(throw) => throw,
            _ => Self::new(err, realm).into(),
        }
    }

    #[must_use]
    pub fn new_from(message: YSString, realm: &Realm) -> ObjectHandle {
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
        let proto = match &error.kind {
            ErrorKind::Type(_) => realm.intrinsics.type_error.clone(),
            ErrorKind::Reference(_) => realm.intrinsics.reference_error.clone(),
            ErrorKind::Range(_) => realm.intrinsics.range_error.clone(),
            ErrorKind::Syntax(_) => realm.intrinsics.syntax_error.clone(),
            ErrorKind::Eval(_) => realm.intrinsics.eval_error.clone(),
            ErrorKind::URI(_) => realm.intrinsics.uri_error.clone(),
            _ => realm.intrinsics.error.clone(),
        };

        Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(proto.into()),
                error,
            }),
        }
    }

    #[must_use]
    pub fn raw_from(message: YSString, realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.error.clone().into()),
                error: Error::unknown_error(message),
            }),
        }
    }

    pub fn override_to_string(&self, _: &mut Realm) -> Res<YSString> {
        let inner = self.inner.try_borrow()?;
        Ok(inner.error.to_ys_string())
    }

    pub fn override_to_string_internal(&self) -> Res<YSString> {
        let inner = self.inner.try_borrow()?;
        Ok(inner.error.to_ys_string())
    }
}

impl CustomName for ErrorObj {
    fn custom_name(&self) -> String {
        "Error".to_string()
    }
}

#[props]
impl ErrorObj {
    #[prop("name")]
    const NAME: &'static str = "Error";

    #[constructor]
    pub fn construct(message: YSString, #[realm] realm: &mut Realm) -> ValueResult {
        let obj = Self::new(Error::unknown_error(message), realm).into();

        Ok(obj)
    }

    #[get("message")]
    pub fn get_message(&self, #[realm] realm: &mut Realm) -> ValueResult {
        let inner = self.inner.try_borrow()?;
        Ok(inner.error.message(realm)?.into())
    }

    #[prop("isError")]
    pub fn is_error(that: Value) -> bool {
        let Value::Object(this) = that else {
            return false;
        };
        
        
        this.downcast::<Self>().is_some()
    }
}
