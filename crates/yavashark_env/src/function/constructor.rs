use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Formatter};

use yavashark_macro::object;
use yavashark_value::{Constructor, Func};

use crate::realm::Realm;
use crate::{Error, MutObject, ObjectHandle, ObjectProperty, Result, Value, ValueResult};

pub type ConstructorFn = Box<dyn Fn(Vec<Value>, &mut Realm) -> ValueResult>;

#[object(function, constructor, direct(constructor))]
pub struct NativeConstructor {
    /// The name of the constructor
    pub name: String,
    /// The function that is called when the constructor is called
    pub f: ConstructorFn,
    #[gc]
    /// The prototype of the constructor
    pub proto: Value,
    /// Can this constructor be called without `new`?
    pub special: bool,
}

impl Debug for NativeConstructor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NativeConstructor({})", self.name)
    }
}

impl Constructor<Realm> for NativeConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        (self.f)(args, realm)
    }

    fn construct_proto(&self) -> Result<ObjectProperty> {
        Ok(self.proto.clone().into())
    }
}

impl Func<Realm> for NativeConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _: Value) -> ValueResult {
        if self.special {
            (self.f)(args, realm)?;

            Ok(Value::Undefined)
        } else {
            Err(Error::ty_error(format!(
                "Constructor {} requires 'new'",
                self.name
            )))
        }
    }
}

impl NativeConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: String,
        f: impl Fn(Vec<Value>, &mut Realm) -> ValueResult + 'static,
        realm: &Realm,
    ) -> ObjectHandle {
        Self::with_proto(
            name,
            f,
            realm.intrinsics.func.clone().into(),
            realm.intrinsics.func.clone().into(),
        )
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn with_proto(
        name: String,
        f: impl Fn(Vec<Value>, &mut Realm) -> ValueResult + 'static,
        proto: Value,
        self_proto: Value,
    ) -> ObjectHandle {
        let this = Self {
            inner: RefCell::new(MutableNativeConstructor {
                object: MutObject::with_proto(self_proto),
                constructor: Value::Undefined.into(),
            }),
            name,
            f: Box::new(f),
            proto,
            special: false,
        };

        let handle = ObjectHandle::new(this);

        #[allow(clippy::expect_used)]
        {
            let constructor = handle.clone();
            let this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor.into();
        }

        handle
    }

    pub fn special(
        name: String,
        f: impl Fn(Vec<Value>, &mut Realm) -> ValueResult + 'static,
        realm: &Realm,
    ) -> ObjectHandle {
        Self::special_with_proto(
            name,
            f,
            realm.intrinsics.func.clone().into(),
            realm.intrinsics.func.clone().into(),
        )
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn special_with_proto(
        name: String,
        f: impl Fn(Vec<Value>, &mut Realm) -> ValueResult + 'static,
        proto: Value,
        self_proto: Value,
    ) -> ObjectHandle {
        let this = Self {
            inner: RefCell::new(MutableNativeConstructor {
                object: MutObject::with_proto(self_proto),
                constructor: Value::Undefined.into(),
            }),
            name,
            f: Box::new(f),
            proto,
            special: true,
        };

        let handle = ObjectHandle::new(this);

        #[allow(clippy::expect_used)]
        {
            let constructor = handle.clone();
            let this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor.into();
        }

        handle
    }
}
