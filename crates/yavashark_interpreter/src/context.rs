use swc_ecma_ast::{Script, Stmt};
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::RuntimeResult;
use crate::scope::Scope;

pub struct Context {
    //TODO: figure out, what needs to be here
}


impl Context {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn run_statements(&mut self, script: &Vec<Stmt>, scope: &mut Scope) -> RuntimeResult {
        let mut last_value = Value::Undefined;
        for stmt in script {
            last_value = self.run_statement(stmt, scope)?;
        }
        
        Ok(last_value)
    }
    

}



