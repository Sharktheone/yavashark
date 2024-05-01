use crate::scope::Scope;
use crate::RuntimeResult;
use swc_ecma_ast::{Script, Stmt};
use yavashark_value::Ctx;
use crate::{Error, Value};


#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    //TODO: figure out, what needs to be here
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