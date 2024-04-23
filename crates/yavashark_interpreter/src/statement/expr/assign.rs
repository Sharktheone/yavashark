use swc_ecma_ast::assignExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_assign(&mut self, stmt: &assignExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}