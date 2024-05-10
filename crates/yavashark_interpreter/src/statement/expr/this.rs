use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ThisExpr;

impl Context {
    pub fn run_this(&mut self, stmt: &ThisExpr, scope: &mut Scope) -> RuntimeResult {
        let this = scope.this.copy();
        Ok(scope.this.copy())
    }
}
