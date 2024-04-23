use swc_ecma_ast::AwaitExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_await(&mut self, stmt: &AwaitExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}