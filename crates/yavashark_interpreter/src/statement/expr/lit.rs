use swc_ecma_ast::Lit;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_lit(&mut self, stmt: &Lit, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}