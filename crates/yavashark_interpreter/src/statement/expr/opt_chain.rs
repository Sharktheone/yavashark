use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::OptChainExpr;

impl Context {
    pub fn run_opt_chain(&mut self, stmt: &OptChainExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
