use crate::realm::Realm;
use crate::{Error, MutObject, NativeConstructor, ObjectHandle, Result, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties};
use yavashark_value::{ErrorKind, Obj};

pub fn get_error(realm: &Realm) -> ValueResult {
    let constr = NativeConstructor::special(
        "error".to_string(),
        |args, realm| {
            let message = args
                .first()
                .map_or(String::new(), std::string::ToString::to_string);

            let obj: Value = ErrorObj::new(Error::unknown_error(message), realm).into();

            Ok(obj)
        },
        realm,
    );
    
    realm.intrinsics.error.define_property("constructor".into(), constr.clone().into())?;
    
    constr.define_property("prototype".into(), realm.intrinsics.error.clone().into())?;
    
    
    Ok(constr.into())
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
        let proto = match &error.kind {
            ErrorKind::Type(_) => realm.intrinsics.type_error.clone(),
            ErrorKind::Reference(_) => realm.intrinsics.reference_error.clone(),
            ErrorKind::Range(_) => realm.intrinsics.range_error.clone(),
            ErrorKind::Syntax(_) => realm.intrinsics.syntax_error.clone(),
            ErrorKind::EvalError(_) => realm.intrinsics.eval_error.clone(),
            ErrorKind::URIError(_) => realm.intrinsics.uri_error.clone(),
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
        let proto = match &error.kind {
            ErrorKind::Type(_) => realm.intrinsics.type_error.clone(),
            ErrorKind::Reference(_) => realm.intrinsics.reference_error.clone(),
            ErrorKind::Range(_) => realm.intrinsics.range_error.clone(),
            ErrorKind::Syntax(_) => realm.intrinsics.syntax_error.clone(),
            ErrorKind::EvalError(_) => realm.intrinsics.eval_error.clone(),
            ErrorKind::URIError(_) => realm.intrinsics.uri_error.clone(),
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
