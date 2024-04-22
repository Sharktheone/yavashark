mod context;
mod scope;
mod statement;

use swc_ecma_ast::Script;
use yavashark_value::error::Error;
use yavashark_value::Value;

pub struct Interpreter {
    script: Script,
}


impl Interpreter {
    pub fn new(script: Script) -> Self {
        Self {
            script,
        }
    }

    pub fn run(&self) -> Result<Value, Error> {
        let mut context = context::Context::new();
        let mut scope = scope::Scope::new();
        context.run_script(&self.script, &mut scope)
    }
}

