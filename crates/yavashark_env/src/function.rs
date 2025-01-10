pub use class::*;
pub use constructor::*;
pub use prototype::*;
use std::cell::{Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use yavashark_macro::custom_props;
use yavashark_value::{MutObj, Obj, ObjectImpl};

use crate::object::Object;
use crate::realm::Realm;
use crate::{Error, MutObject, ObjectHandle, ObjectProperty, Value, ValueResult};

mod bound;
mod class;
mod constructor;
mod prototype;

type NativeFn = Box<dyn Fn(Vec<Value>, Value, &mut Realm) -> ValueResult>;

pub struct NativeFunctionBuilder(NativeFunction, bool);

struct MutNativeFunction {
    pub object: MutObject,
    pub constructor: ObjectProperty,
}

pub struct NativeFunction {
    pub name: String,
    pub f: NativeFn,
    pub special_constructor: bool,
    inner: RefCell<MutNativeFunction>,
}

#[custom_props(constructor)]
impl ObjectImpl<Realm> for NativeFunction {
    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj<Realm>> {
        RefMut::map(self.inner.borrow_mut(), |inner| &mut inner.object)
    }

    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        (self.f)(args, this, realm)
    }

    fn get_constructor_value(&self, realm: &mut Realm) -> Result<Option<Value>, Error> {
        Ok(Some(Object::new(realm).into()))
    }

    fn get_constructor_proto(&self, _realm: &mut Realm) -> Result<Option<Value>, Error> {
        let inner = self.inner.borrow();

        Ok(Some(inner.constructor.value.copy())) //TODO: this is not correct (i think)
    }

    fn special_constructor(&self) -> bool {
        self.special_constructor
    }
}

impl NativeFunction {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_boxed(name: String, f: NativeFn, realm: &Realm) -> ObjectHandle {
        let this = Self {
            name,
            f,
            special_constructor: false,

            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
                constructor: ObjectProperty::new(Value::Undefined.into()),
            }),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::new_ret_no_self, clippy::missing_panics_doc)]
    pub fn new(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        realm: &Realm,
    ) -> ObjectHandle {
        let this = Self {
            name: name.to_string(),
            f: Box::new(f),
            special_constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
                constructor: ObjectProperty::new(Value::Undefined.into()),
            }),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::new_ret_no_self, clippy::missing_panics_doc)]
    pub fn special(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        realm: &Realm,
    ) -> ObjectHandle {
        let this = Self {
            name: name.to_string(),
            f: Box::new(f),
            special_constructor: true,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
                constructor: ObjectProperty::new(Value::Undefined.into()),
            }),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let mut this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn with_proto(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        proto: Value,
    ) -> ObjectHandle {
        let this = Self {
            name: name.to_string(),
            f: Box::new(f),
            special_constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(proto),
                constructor: ObjectProperty::new(Value::Undefined.into()),
            }),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let mut this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn special_with_proto(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        proto: Value,
    ) -> ObjectHandle {
        let this = Self {
            name: name.to_string(),
            f: Box::new(f),
            special_constructor: true,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(proto),
                constructor: ObjectProperty::new(Value::Undefined.into()),
            }),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let mut this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[must_use]
    pub fn builder() -> NativeFunctionBuilder {
        NativeFunctionBuilder(
            Self {
                name: String::new(),
                f: Box::new(|_, _, _| Ok(Value::Undefined)),
                special_constructor: false,
                inner: RefCell::new(MutNativeFunction {
                    object: MutObject::with_proto(Value::Undefined),
                    constructor: ObjectProperty::new(Value::Undefined.into()),
                }),
            },
            true,
        )
    }
}

impl NativeFunctionBuilder {
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.0.name = name.to_string();
        self
    }

    #[must_use]
    pub fn func(mut self, f: NativeFn) -> Self {
        self.0.f = f;
        self
    }

    #[must_use]
    pub fn boxed_func(
        mut self,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
    ) -> Self {
        self.0.f = Box::new(f);
        self
    }

    /// Note: Overwrites a potential prototype that was previously set
    #[must_use]
    pub fn object(self, object: MutObject) -> Self {
        let mut inner = self.0.inner.borrow_mut();
        inner.object = object;

        drop(inner);

        self
    }

    /// Note: Overwrites a potential object that was previously set
    #[must_use]
    pub fn proto(self, proto: Value) -> Self {
        let mut inner = self.0.inner.borrow_mut();

        inner.object.prototype = proto.into();
        drop(inner);

        self
    }

    /// Note: Overrides the prototype of the object
    #[must_use]
    pub fn context(self, realm: &Realm) -> Self {
        let mut inner = self.0.inner.borrow_mut();

        inner.object.prototype = realm.intrinsics.func.clone().into();
        drop(inner);

        self
    }

    #[must_use]
    pub const fn special_constructor(mut self, special: bool) -> Self {
        self.0.special_constructor = special;
        self
    }

    #[must_use]
    pub fn constructor(mut self, constructor: Value) -> Self {
        let mut inner = self.0.inner.borrow_mut();

        inner.constructor = constructor.into();
        drop(inner);

        self.1 = false;
        self
    }

    /// Builds the function handle.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn build(self) -> ObjectHandle {
        let handle = ObjectHandle::new(self.0);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let mut this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<NativeFunction>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Function: {}]", self.name)
    }
}
