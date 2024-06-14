use swc_ecma_ast::ThisExpr;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_this(&mut self, stmt: &ThisExpr, scope: &mut Scope) -> RuntimeResult {
        Ok(scope.this()?.copy())
    }
}
