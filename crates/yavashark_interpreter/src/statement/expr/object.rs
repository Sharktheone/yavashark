use swc_ecma_ast::objectExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_object(&mut self, stmt: &objectExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}