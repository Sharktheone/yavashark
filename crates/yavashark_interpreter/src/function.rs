use std::any::Any;
use std::fmt::Debug;

use swc_ecma_ast::{BlockStmt, Param, Pat};

pub use prototype::*;
use yavashark_macro::object;
use yavashark_value::Func;
use yavashark_value::Obj;

use crate::context::Context;
use crate::object::Object;
use crate::scope::Scope;
use crate::{ControlFlow, Error, FunctionHandle, Value, ValueResult, Variable};

mod prototype;

type NativeFn = Box<dyn FnMut(Vec<Value>, Value, &mut Context) -> ValueResult>;

pub struct NativeFunctionBuilder(NativeFunction);

#[object]
pub struct NativeFunction {
    pub name: String,
    pub f: NativeFn,
    pub data: Option<Box<dyn Any>>,
    // pub prototype: ConstructorPrototype,
}

impl NativeFunction {
    pub fn new_boxed(name: String, f: NativeFn, ctx: &mut Context) -> FunctionHandle {
        let this: Box<dyn Func<Context>> = Box::new(Self {
            name,
            f,
            object: Object::raw_with_proto(ctx.proto.func.clone().into()),
            data: None,
        });

        this.into()
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Context) -> ValueResult + 'static,
        ctx: &mut Context,
    ) -> FunctionHandle {
        let this: Box<dyn Func<Context>> = Box::new(Self {
            name: name.to_string(),
            f: Box::new(f),
            object: Object::raw_with_proto(ctx.proto.func.clone().into()),
            data: None,
        });

        this.into()
    }

    pub fn with_proto(
        name: &str,
        f: impl Fn(Vec<Value>, Value, &mut Context) -> ValueResult + 'static,
        proto: Value,
    ) -> FunctionHandle {
        let this: Box<dyn Func<Context>> = Box::new(Self {
            name: name.to_string(),
            f: Box::new(f),
            object: Object::raw_with_proto(proto),
            data: None,
        });

        this.into()
    }

    #[must_use]
    pub fn builder() -> NativeFunctionBuilder {
        NativeFunctionBuilder(Self {
            name: String::new(),
            f: Box::new(|_, _, _| Ok(Value::Undefined)),
            object: Object::raw_with_proto(Value::Undefined),
            data: None,
        })
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
    pub fn context(mut self, ctx: &mut Context) -> Self {
        self.0.object.prototype = ctx.proto.func.clone().into();
        self
    }

    // Sets the data that can be accessed by the function
    #[must_use]
    pub fn data(mut self, data: Box<dyn Any>) -> Self {
        self.0.data = Some(data);
        self
    }

    /// Builds the function handle.
    #[must_use]
    pub fn build(self) -> FunctionHandle {
        let this: Box<dyn Func<Context>> = Box::new(self.0);
        this.into()
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

#[allow(clippy::module_name_repetitions)]
#[object]
#[derive(Debug)]
pub struct JSFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub block: Option<BlockStmt>,
    pub scope: Scope,
}

impl JSFunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: String,
        params: Vec<Param>,
        block: Option<BlockStmt>,
        scope: Scope,
        ctx: &mut Context,
    ) -> FunctionHandle {
        let this: Box<dyn Func<Context>> = Box::new(Self {
            name,
            params,
            block,
            scope,
            object: Object::raw_with_proto(ctx.proto.func.clone().into()),
        });

        this.into()
    }
}

impl Func<Context> for JSFunction {
    fn call(&mut self, ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope);
        for (i, p) in self.params.iter().enumerate() {
            let Pat::Ident(name) = &p.pat else {
                return Err(Error::syn("Invalid function parameter"));
            };

            scope.declare_var(
                name.sym.to_string(),
                args.get(i).unwrap_or(&Value::Undefined).copy(),
            );
        }

        if let Some(block) = &self.block {
            if let Err(e) = ctx.run_block_this(block, scope, this) {
                return match e {
                    ControlFlow::Error(e) => Err(e),
                    ControlFlow::Return(v) => Ok(v),
                    ControlFlow::Break(_) => Err(Error::syn("Illegal break statement")),
                    ControlFlow::Continue(_) => Err(Error::syn("Illegal continue statement")),
                };
            }
        }
        Ok(Value::Undefined)
    }
}
