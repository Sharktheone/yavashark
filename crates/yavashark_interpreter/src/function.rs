mod prototype;

pub use prototype::*;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

use swc_ecma_ast::{BlockStmt, Param, Pat};

use yavashark_value::Func;
use yavashark_value::Obj;

use crate::context::Context;
use crate::object::Object;
use crate::scope::Scope;
use crate::{ControlFlow, Error, FunctionHandle, ObjectHandle, Value, ValueResult, Variable};

type NativeFn = Box<dyn FnMut(Vec<Value>, Value) -> ValueResult>;

pub struct NativeFunctionBuilder(NativeFunction);

pub struct NativeFunction {
    pub name: String,
    pub f: NativeFn,
    pub object: Object,
    pub data: Option<Box<dyn Any>>,
}

impl NativeFunction {
    pub fn new_boxed(name: String, f: NativeFn, ctx: &mut Context) -> FunctionHandle {
        let this: Box<dyn Func<Context>> = Box::new(Self {
            name,
            f,
            object: Object::raw_with_proto(ctx.func_prototype.clone().into()),
            data: None,
        });

        this.into()
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: &str,
        f: impl Fn(Vec<Value>, Value) -> ValueResult + 'static,
        ctx: &mut Context,
    ) -> FunctionHandle {
        let this: Box<dyn Func<Context>> = Box::new(Self {
            name: name.to_string(),
            f: Box::new(f),
            object: Object::raw_with_proto(ctx.func_prototype.clone().into()),
            data: None,
        });

        this.into()
    }

    pub fn with_proto(
        name: &str,
        f: impl Fn(Vec<Value>, Value) -> ValueResult + 'static,
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

    pub fn builder() -> NativeFunctionBuilder {
        NativeFunctionBuilder(NativeFunction {
            name: "".to_string(),
            f: Box::new(|_, _| Ok(Value::Undefined)),
            object: Object::raw_with_proto(Value::Undefined),
            data: None,
        })
    }
}

impl NativeFunctionBuilder {
    pub fn name(mut self, name: &str) -> Self {
        self.0.name = name.to_string();
        self
    }

    pub fn func(mut self, f: NativeFn) -> Self {
        self.0.f = f;
        self
    }

    pub fn boxed_func(mut self, f: impl Fn(Vec<Value>, Value) -> ValueResult + 'static) -> Self {
        self.0.f = Box::new(f);
        self
    }

    /// Note: Overwrites a potential prototype that was previously set
    pub fn object(mut self, object: Object) -> Self {
        self.0.object = object;
        self
    }

    /// Note: Overwrites a potential object that was previously set
    pub fn proto(mut self, proto: Value) -> Self {
        self.0.object.prototype = proto.into(); //TODO: this doesn't work when you want to also set an object
        self
    }

    /// Note: Overrides the prototype of the object
    pub fn context(mut self, ctx: &mut Context) -> Self {
        self.0.object.prototype = ctx.func_prototype.clone().into();
        self
    }

    // Sets the data that can be accessed by the function
    pub fn data(mut self, data: Box<dyn Any>) -> Self {
        self.0.data = Some(data);
        self
    }

    /// Builds the function handle.
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

impl Obj<Context> for NativeFunction {
    fn define_property(&mut self, name: Value, value: Value) {
        self.object.define_property(name, value);
    }

    fn define_variable(&mut self, name: Value, value: Variable) {
        self.object.define_variable(name, value);
    }

    fn resolve_property(&self, name: &Value) -> Option<Value> {
        self.object.resolve_property(name)
    }

    fn get_property(&self, name: &Value) -> Option<&Value> {
        self.object.get_property(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        self.object.get_property_mut(name)
    }

    fn contains_key(&self, name: &yavashark_value::Value<Context>) -> bool {
        self.object.contains_key(name)
    }

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn to_string(&self) -> String {
        format!("[Function: {}() {{ [Native code] }}]", self.name)
    }

    fn properties(&self) -> Vec<(Value, Value)> {
        self.object.properties()
    }

    fn keys(&self) -> Vec<Value> {
        self.object.keys()
    }

    fn values(&self) -> Vec<Value> {
        self.object.values()
    }
}

impl Func<Context> for NativeFunction {
    fn call(&mut self, _ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
        (self.f)(args, this)
    }
}

#[derive(Debug)]
pub struct JSFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub block: Option<BlockStmt>,
    pub scope: Scope,
    pub object: Object,
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
            object: Object::raw_with_proto(ctx.func_prototype.clone().into()),
        });

        this.into()
    }
}

impl Obj<Context> for JSFunction {
    fn define_property(&mut self, name: Value, value: Value) {
        self.object.define_property(name, value);
    }

    fn define_variable(&mut self, name: Value, value: Variable) {
        self.object.define_variable(name, value)
    }

    fn resolve_property(&self, name: &Value) -> Option<Value> {
        self.object.resolve_property(name).map(|v| v.copy())
    }

    fn get_property(&self, name: &Value) -> Option<&Value> {
        self.object.get_property(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        self.object.get_property_mut(name)
    }

    fn contains_key(&self, name: &yavashark_value::Value<Context>) -> bool {
        self.object.contains_key(name)
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn to_string(&self) -> String {
        format!("[Function: {}() {{ [JS code] }}]", self.name)
    }

    fn properties(&self) -> Vec<(Value, Value)> {
        self.object
            .properties()
            .iter()
            .map(|(k, v)| (k.copy(), v.copy()))
            .collect()
    }

    fn keys(&self) -> Vec<Value> {
        self.object.keys()
    }

    fn values(&self) -> Vec<Value> {
        self.object.values()
    }
}

impl Func<Context> for JSFunction {
    fn call(&mut self, ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope);
        for (i, p) in self.params.iter().enumerate() {
            let name = match p.pat {
                Pat::Ident(ref i) => i.sym.to_string(),
                _ => todo!("call args pat"),
            };

            scope.declare_var(name, args.get(i).unwrap_or(&Value::Undefined).copy());
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
