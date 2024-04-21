use swc_ecma_ast::Script;
use crate::scope::Scope;

pub struct Context {
    //TODO: figure out, what needs to be here
}


impl Context {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn run_script(&mut self, script: &Script, scope: &mut Scope) {
        todo!()
    }

}



