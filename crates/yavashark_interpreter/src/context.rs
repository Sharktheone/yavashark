use std::cell::{Ref, RefCell};
use std::rc::Rc;
use crate::scope::Scope;
use crate::{Object, RuntimeResult};
use swc_ecma_ast::{Script, Stmt};
use yavashark_value::{Ctx, Obj};
use crate::{Error, Value};
use crate::object::Prototype;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    pub(crate) obj_prototype: Object,
}


impl Default for Context {
    fn default() -> Self {
        let obj_prototype: Box<dyn Obj<Context>> = Box::new(Prototype::new());
        let obj_prototype = Object::new(obj_prototype);
        Self { obj_prototype }
    }
}

impl Context {
    
    
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn run_statements(&mut self, script: &Vec<Stmt>, scope: &mut Scope) -> RuntimeResult {
        let mut last_value = Value::Undefined;
        for stmt in script {
            last_value = self.run_statement(stmt, scope)?;
        }

        Ok(last_value)
    }
}


impl Ctx for Context {}