use swc_ecma_ast::OptChainExpr;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_opt_chain(&mut self, stmt: &OptChainExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
