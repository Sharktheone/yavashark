mod context;
mod scope;
mod statement;

use swc_ecma_ast::Script;

pub struct Interpreter {
    script: Script,
}


impl Interpreter {
    pub fn new(script: Script) -> Self {
        Self {
            script,
        }
    }

    pub fn run(&self) {
        let mut context = context::Context::new();
        let mut scope = scope::Scope::new();
        context.run_script(&self.script, &mut scope);
    }
}

