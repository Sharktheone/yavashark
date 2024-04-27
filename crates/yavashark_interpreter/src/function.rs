use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use crate::scope::Scope;
use crate::{Object, Res, RuntimeResult, Value, ValueResult};


pub enum Function {
    Native(Box<dyn FnMut(Vec<Value>, &mut Scope) -> ValueResult>),
}


impl Function {
    pub fn call(&mut self, args: Vec<Value>, scope: &mut Scope) -> ValueResult {
        match self {
            Function::Native(f) => f(args, scope),
        }
    }


    pub fn native(f: Box<dyn FnMut(Vec<Value>, &mut Scope) -> ValueResult>) -> Self {
        Function::Native(f)
    }
    
    pub fn native_val(f: Box<dyn FnMut(Vec<Value>, &mut Scope) -> ValueResult>) -> Value {
        let obj = Function::native(f).into();
        let ohj = Rc::new(RefCell::new(obj));
        Value::Object(ohj)
    }
}


impl Into<Object> for Function {
    fn into(self) -> Object {
        let mut obj = Object::new();
        obj.call = Some(self);
        obj
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Function]")
    }
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        false //TODO
    }
}