use swc_ecma_ast::CallExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_call(&mut self, stmt: &CallExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}