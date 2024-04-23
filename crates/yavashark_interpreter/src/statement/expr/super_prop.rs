use swc_ecma_ast::SuperPropExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_super_prop(&mut self, stmt: &SuperPropExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}