use swc_ecma_ast::ParenExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_paren(&mut self, stmt: &ParenExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}