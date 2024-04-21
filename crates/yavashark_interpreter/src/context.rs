use swc_ecma_ast::{Script, Stmt};
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::scope::Scope;

pub struct Context {
    //TODO: figure out, what needs to be here
}


impl Context {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn run_script(&mut self, script: &Script, scope: &mut Scope) -> Result<(), Error> {
        for stmt in &script.body {
            self.run_statement(stmt, scope)?;
        }
        
        Ok(())
    }
    

}



