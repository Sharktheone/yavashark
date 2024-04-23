use swc_ecma_ast::memberExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_member(&mut self, stmt: &memberExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}