use swc_ecma_ast::ThisExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_this(&mut self, stmt: &ThisExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}