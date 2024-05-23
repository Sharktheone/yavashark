use swc_ecma_ast::YieldExpr;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_yield(&mut self, stmt: &YieldExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
