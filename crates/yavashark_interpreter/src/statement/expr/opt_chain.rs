use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::OptChainExpr;
use yavashark_value::error::Error;

impl Context {
    pub fn run_opt_chain(&mut self, stmt: &OptChainExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
