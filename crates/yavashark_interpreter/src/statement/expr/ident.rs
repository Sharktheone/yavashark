use swc_ecma_ast::identExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_ident(&mut self, stmt: &identExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}