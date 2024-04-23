use swc_ecma_ast::YieldExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_yield(&mut self, stmt: &YieldExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}