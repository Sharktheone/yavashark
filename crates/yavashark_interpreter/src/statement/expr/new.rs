use swc_ecma_ast::newExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_new(&mut self, stmt: &newExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}