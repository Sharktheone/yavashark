mod prototype;

use std::collections::HashMap;
use std::fmt::Debug;

use swc_ecma_ast::{BlockStmt, Param, Pat};

use yavashark_value::Func;
use yavashark_value::Obj;

use crate::{ControlFlow, Error, Function, Value, ValueResult};
use crate::context::Context;
use crate::scope::Scope;



type NativeFn = Box<dyn FnMut(Vec<Value>, Value) -> ValueResult>;


pub struct NativeFunction {
    pub name: String,
    pub f: NativeFn,
    pub properties: HashMap<Value, Value>,
}

impl NativeFunction {
    pub fn new_boxed(name: String, f: NativeFn) -> Function {
        let this: Box<dyn Func<Context>> = Box::new(Self {
            name,
            f,
            properties: HashMap::new(),
        });

        this.into()
    }
    
    #[allow(clippy::new_ret_no_self)]
    pub fn new(name: &str, f: impl Fn(Vec<Value>, Value) -> ValueResult + 'static) -> Function {
        let this: Box<dyn Func<Context>> = Box::new(Self {
            name: name.to_string(),
            f: Box::new(f),
            properties: HashMap::new(),
        });

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
        self.properties.insert(name, value);
    }

    fn resolve_property(&self, name: &Value) -> Option<Value> {
        self.properties.get(name).map(|v| v.copy())
    }


    fn get_property(&self, name: &Value) -> Option<&Value> {
        self.properties.get(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        self.properties.get_mut(name)
    }

    fn contains_key(&self, name: &yavashark_value::Value<Context>) -> bool {
        self.properties.contains_key(name)
    }

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn to_string(&self) -> String {
        format!("[Function: {}() {{ [Native code] }}]", self.name)
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
    pub properties: HashMap<Value, Value>,
}

impl JSFunction {
    pub fn new(name: String, params: Vec<Param>, block: Option<BlockStmt>, scope: Scope) -> Function {
        let this: Box<dyn Func<Context>> = Box::new(Self {
            name,
            params,
            block,
            scope,
            properties: HashMap::new(),
        });

        this.into()
    }
}

impl Obj<Context> for JSFunction {
    fn define_property(&mut self, name: Value, value: Value) {
        self.properties.insert(name, value);
    }


    fn resolve_property(&self, name: &Value) -> Option<Value> {
        self.properties.get(name).map(|v| v.copy())
    }
    
    fn get_property(&self, name: &Value) -> Option<&Value> {
        self.properties.get(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        self.properties.get_mut(name)
    }

    fn contains_key(&self, name: &yavashark_value::Value<Context>) -> bool {
        self.properties.contains_key(name)
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn to_string(&self) -> String {
        format!("[Function: {}() {{ [JS code] }}]", self.name)
    }
}


impl Func<Context> for JSFunction {
    fn call(&mut self, ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
        let scope = &mut Scope::with_parent(&self.scope);
        for (i, p) in self.params.iter().enumerate() {
            let name = match p.pat {
                Pat::Ident(ref i) => i.sym.to_string(),
                _ => todo!("call args pat")
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