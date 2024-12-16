use std::fmt::Debug;

pub use class::*;
pub use constructor::*;
pub use prototype::*;
use yavashark_macro::object;
use yavashark_value::{Constructor, Error, Func, Obj, ObjectImpl};

use crate::object::Object;
use crate::realm::Realm;
use crate::{ObjectHandle, Value, ValueResult, ObjectProperty};

mod bound;
mod class;
mod constructor;
mod prototype;

type NativeFn = Box<dyn FnMut(Vec<Value>, Value, &mut Realm) -> ValueResult>;

pub struct NativeFunctionBuilder(NativeFunction, bool);

pub struct NativeFunction {
    pub name: String,
    pub f: NativeFn,
    pub special_constructor: bool,
    pub constructor: ObjectProperty,
    pub object: Object,
    // pub prototype: ConstructorPrototype,
}


//#[custom_props(constructor)] TODO
impl ObjectImpl<Realm> for NativeFunction {
    fn get_wrapped_object(&self) -> &impl Obj<Realm> {
        &self.object
    }

    fn get_wrapped_object_mut(&mut self) -> &mut impl Obj<Realm> {
        &mut self.object
    }


    fn call(&mut self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        (self.f)(args, this, realm)
    }

    fn get_constructor_value(&self, realm: &mut Realm) -> Option<yavashark_value::Value<Realm>> {
        Some(Object::new(realm).into())
    }

    fn get_constructor_proto(&self, realm: &mut Realm) -> Option<Value> {
        Some(self.constructor.value.copy()) //TODO: this is not correct (i think)
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
            object: Object::raw_with_proto(realm.intrinsics.func.clone().into()),
            special_constructor: false,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let mut this = handle.get_mut().expect("unreachable");

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().expect("unreachable");

            this.constructor = constructor;
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
            object: Object::raw_with_proto(realm.intrinsics.func.clone().into()),
            special_constructor: false,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let mut this = handle.get_mut().expect("unreachable");

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().expect("unreachable");

            this.constructor = constructor;
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
            object: Object::raw_with_proto(realm.intrinsics.func.clone().into()),
            special_constructor: true,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let mut this = handle.get_mut().expect("unreachable");

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().expect("unreachable");

            this.constructor = constructor;
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
            object: Object::raw_with_proto(proto),
            special_constructor: false,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let mut this = handle.get_mut().expect("unreachable");

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().expect("unreachable");

            this.constructor = constructor;
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
            object: Object::raw_with_proto(proto),
            special_constructor: true,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        #[allow(clippy::expect_used)]
        {
            let mut this = handle.get_mut().expect("unreachable");

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().expect("unreachable");

            this.constructor = constructor;
        }

        handle
    }

    #[must_use]
    pub fn builder() -> NativeFunctionBuilder {
        NativeFunctionBuilder(
            Self {
                name: String::new(),
                f: Box::new(|_, _, _| Ok(Value::Undefined)),
                object: Object::raw_with_proto(Value::Undefined),
                special_constructor: false,
                constructor: Value::Undefined.into(),
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
    pub fn object(mut self, object: Object) -> Self {
        self.0.object = object;
        self
    }

    /// Note: Overwrites a potential object that was previously set
    #[must_use]
    pub fn proto(mut self, proto: Value) -> Self {
        self.0.object.prototype = proto.into(); //TODO: this doesn't work when you want to also set an object
        self
    }

    /// Note: Overrides the prototype of the object
    #[must_use]
    pub fn context(mut self, realm: &Realm) -> Self {
        self.0.object.prototype = realm.intrinsics.func.clone().into();
        self
    }

    #[must_use]
    pub const fn special_constructor(mut self, special: bool) -> Self {
        self.0.special_constructor = special;
        self
    }

    #[must_use]
    pub fn constructor(mut self, constructor: Value) -> Self {
        self.0.constructor = constructor.into();
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
            let mut this = handle.get_mut().expect("unreachable");

            let this = this.as_any_mut();

            let this = this.downcast_mut::<NativeFunction>().expect("unreachable");

            this.constructor = constructor;
        }

        handle
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Function: {}]", self.name)
    }
}
