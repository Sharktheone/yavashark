use swc_ecma_ast::condExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_cond(&mut self, stmt: &condExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}