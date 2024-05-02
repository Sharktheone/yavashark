use std::cell::{Ref, RefCell};
use std::rc::Rc;
use crate::scope::Scope;
use crate::RuntimeResult;
use swc_ecma_ast::{Script, Stmt};
use yavashark_value::Ctx;
use crate::{Error, Value};
use crate::object::Prototype;


#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Context {
    obj_prototype: Rc<RefCell<Prototype>>,
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