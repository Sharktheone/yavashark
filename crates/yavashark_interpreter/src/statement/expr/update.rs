use swc_ecma_ast::UpdateExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_update(&mut self, stmt: &UpdateExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}