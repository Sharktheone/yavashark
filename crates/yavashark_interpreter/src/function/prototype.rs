use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::object::Prototype;
use crate::{NativeFunction, Value, ValueResult};


pub struct FunctionPrototype {
    pub properties: HashMap<Value, Value>,
    pub parent: Rc<RefCell<Prototype>>,
    pub apply: Value,
    pub bind: Value,
    pub call: Value,
}


impl Default for FunctionPrototype {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionPrototype {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            parent: Rc::new(RefCell::new(Prototype::new())),
            apply: NativeFunction::new("apply", apply).into(),
            bind: NativeFunction::new("bind", bind).into(),
            call: NativeFunction::new("call", call).into()
        }
    }
}


fn apply(args: Vec<Value>, this: Value) -> ValueResult {
    todo!()
}

fn bind(args: Vec<Value>, this: Value) -> ValueResult {
    todo!()
}

fn call(args: Vec<Value>, this: Value) -> ValueResult {
    todo!()
}