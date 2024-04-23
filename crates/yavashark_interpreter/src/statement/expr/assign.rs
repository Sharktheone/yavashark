use swc_ecma_ast::AssignExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_assign(&mut self, stmt: &AssignExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}