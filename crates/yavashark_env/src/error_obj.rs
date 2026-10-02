use crate::error::ErrorKind;
use crate::realm::Realm;
use crate::value::{CustomName, MutObj};
use crate::{Error, MutObject, ObjectHandle, Res, Value, ValueResult, Variable};
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
    pub fn new(error: Error, realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(ObjectHandle::new(Self::raw(error, realm)?))
    }

    pub fn error_to_value(err: Error, realm: &mut Realm) -> ValueResult {
        Ok(match err.kind {
            ErrorKind::Throw(throw) => throw,
            _ => Self::new(err, realm)?.into(),
        })
    }

    pub fn new_from(message: YSString, realm: &mut Realm) -> Res<ObjectHandle> {
        Self::new(Error::unknown_error(message), realm)
    }

    pub fn raw(error: Error, realm: &mut Realm) -> Res<Self> {
        let message = error.message(realm)?;
        Self::with_message(error, Some(message), realm)
    }

    pub fn with_message(error: Error, message: Option<YSString>, realm: &mut Realm) -> Res<Self> {
        let proto = match &error.kind {
            ErrorKind::Type(_) => realm.intrinsics.clone_public().ty_error.get(realm)?.clone(),
            ErrorKind::Reference(_) => realm
                .intrinsics
                .clone_public()
                .reference_error
                .get(realm)?
                .clone(),
            ErrorKind::Range(_) => realm
                .intrinsics
                .clone_public()
                .range_error
                .get(realm)?
                .clone(),
            ErrorKind::Syntax(_) => realm
                .intrinsics
                .clone_public()
                .syn_error
                .get(realm)?
                .clone(),
            ErrorKind::Eval(_) => realm
                .intrinsics
                .clone_public()
                .eval_error
                .get(realm)?
                .clone(),
            ErrorKind::URI(_) => realm
                .intrinsics
                .clone_public()
                .uri_error
                .get(realm)?
                .clone(),
            ErrorKind::Aggregate(_) => realm
                .intrinsics
                .clone_public()
                .aggregate_error
                .get(realm)?
                .clone(),
            ErrorKind::Suppressed(_) => realm
                .intrinsics
                .clone_public()
                .suppressed_error
                .get(realm)?
                .clone(),
            _ => realm.intrinsics.clone_public().error.get(realm)?.clone(),
        };

        let mut object = MutObject::with_proto(proto);
        if let Some(message) = message {
            object.define_property_attributes(
                "message".into(),
                Variable::write_config(message.into()),
                realm,
            )?;
        }

        Ok(Self {
            inner: RefCell::new(MutableErrorObj { object, error }),
        })
    }

    pub fn raw_from(message: YSString, realm: &mut Realm) -> Res<Self> {
        Self::raw(Error::unknown_error(message), realm)
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
    #[configurable]
    #[writable]
    #[both]
    const NAME: &'static str = "Error";

    #[constructor]
    #[call_constructor]
    pub fn construct(message: Option<YSString>, #[realm] realm: &mut Realm) -> ValueResult {
        let error = Error::unknown_error(message.clone().unwrap_or_default());
        let obj = ObjectHandle::new(Self::with_message(error, message, realm)?).into();

        Ok(obj)
    }

    #[prop("toString")]
    pub fn to_js_string(&self, #[realm] realm: &mut Realm) -> Res<YSString> {
        let inner = self.inner.try_borrow()?;

        let message = inner.error.message(realm)?;
        let name = inner.error.name();

        if message.is_empty() {
            return Ok(name.into());
        }

        Ok(format!("{name}: {message}").into())
    }

    #[prop("message")]
    #[configurable]
    #[writable]
    const MESSAGE: &'static str = "";

    #[prop("isError")]
    pub fn is_error(that: Value) -> bool {
        let Value::Object(this) = that else {
            return false;
        };

        this.downcast::<Self>().is_some()
    }
}
