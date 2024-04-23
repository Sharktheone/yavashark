use swc_ecma_ast::OptChainExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_opt_chain(&mut self, stmt: &OptChainExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}