use swc_ecma_ast::Decl;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_decl(&mut self, stmt: &Decl, scope: &mut Scope) -> Result<Value, Error> {
        todo!()
    }
}
