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
}


impl Into<Object> for Function {
    fn into(self) -> Object {
        let mut obj = Object::new();
        obj.call = Some(self);
        obj
    }
}