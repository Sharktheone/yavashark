use swc_ecma_ast::OptChainExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_opt_chain(&mut self, stmt: &OptChainExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}