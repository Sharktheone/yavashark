use swc_ecma_ast::CondExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_cond(&mut self, stmt: &CondExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}