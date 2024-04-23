use swc_ecma_ast::ArrowExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_arrow(&mut self, stmt: &ArrowExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}