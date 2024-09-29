use std::any::Any;
use std::fmt::Debug;

pub use class::*;
pub use constructor::*;
pub use prototype::*;
use yavashark_macro::object;
use yavashark_value::{Constructor, Func, ObjectProperty};

use crate::context::Context;
use crate::object::Object;
use crate::{ObjectHandle, Value, ValueResult};

mod class;
mod constructor;
mod prototype;

type NativeFn = Box<dyn FnMut(Vec<Value>, Value, &mut Context) -> ValueResult>;

pub struct NativeFunctionBuilder(NativeFunction, bool);

#[object(function, constructor, direct(constructor))]
pub struct NativeFunction {
    pub name: String,
    pub f: NativeFn,
    pub data: Option<Box<dyn Any>>,
    special_constructor: bool,
    // pub prototype: ConstructorPrototype,
}

impl Constructor<Context> for NativeFunction {
    fn get_constructor(&self) -> ObjectProperty<Context> {
        self.constructor.copy()
    }

    fn special_constructor(&self) -> bool {
        self.special_constructor
    }

    fn value(&self, ctx: &mut Context) -> Value {
        Object::new(ctx).into()
    }
}

impl NativeFunction {
    #[must_use]
    pub fn new_boxed(name: String, f: NativeFn, ctx: &Context) -> ObjectHandle {
        let this = Self {
            name,
            f,
            object: Object::raw_with_proto(ctx.proto.func.clone().into()),
            data: None,
            special_constructor: false,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        {
            let mut this = handle.get_mut().unwrap();

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().unwrap();

            this.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Context) -> ValueResult + 'static,
        ctx: &Context,
    ) -> ObjectHandle {
        let this = Self {
            name: name.to_string(),
            f: Box::new(f),
            object: Object::raw_with_proto(ctx.proto.func.clone().into()),
            data: None,
            special_constructor: false,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        {
            let mut this = handle.get_mut().unwrap();

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().unwrap();

            this.constructor = constructor;
        }

        handle
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn special(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Context) -> ValueResult + 'static,
        ctx: &Context,
    ) -> ObjectHandle {
        let this = Self {
            name: name.to_string(),
            f: Box::new(f),
            object: Object::raw_with_proto(ctx.proto.func.clone().into()),
            data: None,
            special_constructor: true,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());


        {
            let mut this = handle.get_mut().unwrap();

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().unwrap();

            this.constructor = constructor;
        }


        handle
    }

    pub fn with_proto(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Context) -> ValueResult + 'static,
        proto: Value,
    ) -> ObjectHandle {
        let this = Self {
            name: name.to_string(),
            f: Box::new(f),
            object: Object::raw_with_proto(proto),
            data: None,
            special_constructor: false,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());


        {
            let mut this = handle.get_mut().unwrap();

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().unwrap();

            this.constructor = constructor;
        }

        handle
    }

    pub fn special_with_proto(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Context) -> ValueResult + 'static,
        proto: Value,
    ) -> ObjectHandle {
        let this = Self {
            name: name.to_string(),
            f: Box::new(f),
            object: Object::raw_with_proto(proto),
            data: None,
            special_constructor: true,
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        let constructor = ObjectProperty::new(handle.clone().into());

        {
            let mut this = handle.get_mut().unwrap();

            let this = this.as_any_mut();

            let this = this.downcast_mut::<Self>().unwrap();

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
                data: None,
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
        f: impl Fn(Vec<Value>, Value, &mut Context) -> ValueResult + 'static,
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
    pub fn context(mut self, ctx: &Context) -> Self {
        self.0.object.prototype = ctx.proto.func.clone().into();
        self
    }

    // Sets the data that can be accessed by the function
    #[must_use]
    pub fn data(mut self, data: Box<dyn Any>) -> Self {
        self.0.data = Some(data);
        self
    }

    #[must_use]
    pub const fn special_constructor(mut self, special: bool) -> Self {
        self.0.special_constructor = special;
        self
    }

    pub fn constructor(mut self, constructor: Value) -> Self {
        self.0.constructor = constructor.into();
        self.1 = false;
        self
    }

    /// Builds the function handle.
    #[must_use]
    pub fn build(self) -> ObjectHandle {
        let handle = ObjectHandle::new(self.0);

        let constructor = ObjectProperty::new(handle.clone().into());

        {
            let mut this = handle.get_mut().unwrap();

            let this = this.as_any_mut();

            let this = this.downcast_mut::<NativeFunction>().unwrap();

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

impl Func<Context> for NativeFunction {
    fn call(&mut self, ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
        (self.f)(args, this, ctx)
    }
}
