use std::cell::RefCell;
use std::fmt;
use std::fmt::{Debug, Formatter};

use crate::value::{Constructor, Func};
use yavashark_macro::object;
use yavashark_string::YSString;

use crate::realm::Realm;
use crate::{Error, MutObject, ObjectHandle, Res, Value, ValueResult, Variable};

pub type ConstructorFn = Box<dyn Fn(Vec<Value>, &mut Realm) -> Res<ObjectHandle>>;

#[object(function, constructor, direct(constructor), to_string)]
pub struct NativeConstructor {
    /// The name of the constructor
    pub name: String,
    /// The function that is called when the constructor is called
    pub f: ConstructorFn,
    #[gc]
    /// The prototype of the constructor
    pub proto: ObjectHandle,
    /// Can this constructor be called without `new`?
    pub special: bool,
}

impl Debug for NativeConstructor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NativeConstructor({})", self.name)
    }
}

impl Constructor for NativeConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> Res<ObjectHandle> {
        (self.f)(args, realm)
    }

    // fn construct_proto(&self) -> Res<ObjectProperty> {
    //     Ok(self.proto.clone().into())
    // }
}

impl Func for NativeConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _: Value) -> ValueResult {
        if self.special {
            Ok((self.f)(args, realm)?.into())
        } else {
            Err(Error::ty_error(format!(
                "Constructor {} requires 'new'",
                self.name
            )))
        }
    }
}

impl NativeConstructor {
    pub fn override_to_string(&self, _: &mut Realm) -> Res<YSString> {
        Ok("function Function() { [native code] }".into())
    }

    pub fn override_to_string_internal(&self) -> Res<YSString> {
        Ok("function Function() { [native code] }".into())
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: String,
        f: impl Fn(Vec<Value>, &mut Realm) -> Res<ObjectHandle> + 'static,
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
        f: impl Fn(Vec<Value>, &mut Realm) -> Res<ObjectHandle> + 'static,
        proto: ObjectHandle,
        self_proto: ObjectHandle,
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

            let this = handle.downcast::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor.into();
        }

        handle
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn with_proto_and_len(
        name: String,
        f: impl Fn(Vec<Value>, &mut Realm) -> Res<ObjectHandle> + 'static,
        proto: ObjectHandle,
        self_proto: ObjectHandle,
        len: usize,
        realm: &mut Realm,
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

        let _ = handle.define_property_attributes(
            "length".into(),
            Variable::config(Value::Number(len as f64)),
            realm,
        );

        #[allow(clippy::expect_used)]
        {
            let constructor = handle.clone();

            let this = handle.downcast::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor.into();
        }

        handle
    }

    pub fn special(
        name: String,
        f: impl Fn(Vec<Value>, &mut Realm) -> Res<ObjectHandle> + 'static,
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
        f: impl Fn(Vec<Value>, &mut Realm) -> Res<ObjectHandle> + 'static,
        proto: ObjectHandle,
        self_proto: ObjectHandle,
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
            let this = handle.downcast::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor.into();
        }

        handle
    }
}
