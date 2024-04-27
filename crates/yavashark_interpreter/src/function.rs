use crate::scope::Scope;
use crate::{Res, RuntimeResult, Value, ValueResult};

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