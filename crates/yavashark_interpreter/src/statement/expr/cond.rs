use swc_ecma_ast::CondExpr;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_cond(&mut self, stmt: &CondExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
