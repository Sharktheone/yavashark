use crate::object::Object;
use crate::realm::Realm;
use crate::{Error, MutObject, ObjectHandle, ObjectProperty, Value, ValueResult, Variable};
pub use class::*;
pub use constructor::*;
pub use prototype::*;
use std::cell::{RefCell, RefMut};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use yavashark_macro::custom_props;
use yavashark_string::YSString;
use crate::value::{MutObj, Obj, ObjectImpl};

mod bound;
mod class;
mod constructor;
mod prototype;

type NativeFn = Box<dyn Fn(Vec<Value>, Value, &mut Realm) -> ValueResult>;

pub struct NativeFunctionBuilder(NativeFunction, bool);

pub struct MutNativeFunction {
    pub object: MutObject,
    pub constructor: ObjectProperty,
}

pub struct NativeFunction {
    pub name: &'static str,
    pub f: NativeFn,
    pub constructor: bool,
    inner: RefCell<MutNativeFunction>,
}

#[custom_props(constructor)]
impl ObjectImpl for NativeFunction {
    type Inner = MutNativeFunction;

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj> {
        RefMut::map(self.inner.borrow_mut(), |inner| &mut inner.object)
    }

    fn get_inner(&self) -> impl Deref<Target = Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner> {
        self.inner.borrow_mut()
    }

    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        (self.f)(args, this, realm)
    }

    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        if !self.constructor {
            return Err(Error::ty_error(format!(
                "{} is not a constructor",
                self.name
            )));
        }

        let proto = Obj::resolve_property(self, &Value::from("prototype".to_string()))?
            .map_or_else(|| realm.intrinsics.func.clone().into(), |p| p.value);

        let obj = Object::with_proto(proto).into();

        (self.f)(args, obj, realm)
    }

    fn to_string(&self, _: &mut Realm) -> Result<YSString, crate::error::Error> {
        Ok(format!("function {}() {{ [native code] }}", self.name).into())
    }

    fn to_string_internal(&self) -> Result<YSString, crate::error::Error> {
        Ok(format!("function {}() {{ [native code] }}", self.name).into())
    }

    fn is_function(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        "Function".to_string()
    }
}

impl NativeFunction {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_boxed(name: &'static str, f: NativeFn, realm: &Realm) -> ObjectHandle {
        let this = Self {
            name,
            f,
            constructor: false,

            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
                constructor: Value::Undefined.into(),
            }),
        };

        let handle = ObjectHandle::new(this);

        let _ = handle.define_variable(
            "name".into(),
            Variable::new_with_attributes(name.into(), false, false, true),
        );

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let this = handle.guard();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::new_ret_no_self, clippy::missing_panics_doc)]
    pub fn new(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        realm: &Realm,
    ) -> ObjectHandle {
        let this = Self {
            name,
            f: Box::new(f),
            constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
                constructor: Value::Undefined.into(),
            }),
        };

        let handle = ObjectHandle::new(this);

        let _ = handle.define_variable(
            "name".into(),
            Variable::new_with_attributes(name.into(), false, false, true),
        );

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let this = handle.guard();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::new_ret_no_self, clippy::missing_panics_doc)]
    pub fn with_len(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        realm: &Realm,
        len: usize,
    ) -> ObjectHandle {
        let this = Self {
            name,
            f: Box::new(f),
            constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
                constructor: Value::Undefined.into(),
            }),
        };

        let handle = ObjectHandle::new(this);
        let _ = handle.define_variable("name".into(), Variable::config(name.into()));
        let _ = handle.define_variable("length".into(), Variable::config(len.into()));

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let this = handle.guard();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::new_ret_no_self, clippy::missing_panics_doc)]
    pub fn special(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        realm: &Realm,
    ) -> ObjectHandle {
        let this = Self {
            name,
            f: Box::new(f),
            constructor: true,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
                constructor: Value::Undefined.into(),
            }),
        };

        let handle = ObjectHandle::new(this);
        let _ = handle.define_variable(
            "name".into(),
            Variable::new_with_attributes(name.into(), false, false, true),
        );

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let this = handle.guard();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn with_proto(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        proto: Value,
    ) -> ObjectHandle {
        let this = Self {
            name,
            f: Box::new(f),
            constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(proto),
                constructor: Value::Undefined.into(),
            }),
        };

        let handle = ObjectHandle::new(this);
        let _ = handle.define_variable("name".into(), Variable::config(name.into()));

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let this = handle.guard();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn with_proto_and_len(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        proto: Value,
        len: usize,
    ) -> ObjectHandle {
        let this = Self {
            name,
            f: Box::new(f),
            constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(proto),
                constructor: Value::Undefined.into(),
            }),
        };

        let handle = ObjectHandle::new(this);
        let _ = handle.define_variable("name".into(), Variable::config(name.into()));
        let _ = handle.define_variable("length".into(), Variable::config(Value::from(len)));

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let this = handle.guard();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            let mut inner = this.inner.borrow_mut();

            inner.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn special_with_proto(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        proto: Value,
    ) -> ObjectHandle {
        let this = Self {
            name,
            f: Box::new(f),
            constructor: true,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(proto),
                constructor: Value::Undefined.into(),
            }),
        };

        let handle = ObjectHandle::new(this);
        let _ = handle.define_variable(
            "name".into(),
            Variable::new_with_attributes(name.into(), false, false, true),
        );

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let this = handle.guard();

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
                name: "",
                f: Box::new(|_, _, _| Ok(Value::Undefined)),
                constructor: false,
                inner: RefCell::new(MutNativeFunction {
                    object: MutObject::with_proto(Value::Undefined),
                    constructor: Value::Undefined.into(),
                }),
            },
            true,
        )
    }
}

impl NativeFunctionBuilder {
    #[must_use]
    pub const fn name(mut self, name: &'static str) -> Self {
        self.0.name = name;
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
        self.0.constructor = special;
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
        let name = self.0.name;
        let handle = ObjectHandle::new(self.0);

        let constructor = ObjectProperty::new(handle.clone().into());
        let _ = handle.define_variable(
            "name".into(),
            Variable::new_with_attributes(name.into(), false, false, true),
        );

        #[allow(clippy::expect_used)]
        {
            let this = handle.guard();

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
