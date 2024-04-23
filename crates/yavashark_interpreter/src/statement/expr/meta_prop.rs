use swc_ecma_ast::MetaPropExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_meta_prop(&mut self, stmt: &MetaPropExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}