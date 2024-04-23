use swc_ecma_ast::FnExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_fn(&mut self, stmt: &FnExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}