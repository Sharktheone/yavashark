use crate::error::ErrorKind;
use crate::realm::Realm;
use crate::value::CustomName;
use crate::{Error, MutObject, ObjectHandle, Res, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, props};
use yavashark_string::{ToYSString, YSString};

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
    pub fn new(error: Error, realm: &mut Realm) -> Res<ObjectHandle> {
        let proto = match &error.kind {
            ErrorKind::Type(_) => realm.intrinsics.type_error.clone(),
            ErrorKind::Reference(_) => realm.intrinsics.reference_error.clone(),
            ErrorKind::Range(_) => realm.intrinsics.range_error.clone(),
            ErrorKind::Syntax(_) => realm.intrinsics.syntax_error.clone(),
            ErrorKind::Eval(_) => realm.intrinsics.eval_error.clone(),
            ErrorKind::URI(_) => realm.intrinsics.uri_error.clone(),
            _ => realm.intrinsics.clone_public().error.get(realm)?.clone(),
        };

        let this = Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(proto),
                error,
            }),
        };

        Ok(ObjectHandle::new(this))
    }

    pub fn error_to_value(err: Error, realm: &mut Realm) -> ValueResult {
        Ok(match err.kind {
            ErrorKind::Throw(throw) => throw,
            _ => Self::new(err, realm)?.into(),
        })
    }

    pub fn new_from(message: YSString, realm: &mut Realm) -> Res<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.clone_public().error.get(realm)?.clone()),
                error: Error::unknown_error(message),
            }),
        };

        Ok(ObjectHandle::new(this))
    }

    pub fn raw(error: Error, realm: &mut Realm) -> Res<Self> {
        let proto = match &error.kind {
            ErrorKind::Type(_) => realm.intrinsics.type_error.clone(),
            ErrorKind::Reference(_) => realm.intrinsics.reference_error.clone(),
            ErrorKind::Range(_) => realm.intrinsics.range_error.clone(),
            ErrorKind::Syntax(_) => realm.intrinsics.syntax_error.clone(),
            ErrorKind::Eval(_) => realm.intrinsics.eval_error.clone(),
            ErrorKind::URI(_) => realm.intrinsics.uri_error.clone(),
            _ => realm.intrinsics.clone_public().error.get(realm)?.clone(),
        };

        Ok(Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(proto),
                error,
            }),
        })
    }

    pub fn raw_from(message: YSString, realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableErrorObj {
                object: MutObject::with_proto(realm.intrinsics.clone_public().error.get(realm)?.clone()),
                error: Error::unknown_error(message),
            }),
        })
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


#[props(intrinsic_name = error)]
impl ErrorObj {
    #[prop("name")]
    #[both]
    const NAME: &'static str = "Error";

    #[constructor]
    pub fn construct(message: YSString, #[realm] realm: &mut Realm) -> ValueResult {
        let obj = Self::new(Error::unknown_error(message), realm)?.into();

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


