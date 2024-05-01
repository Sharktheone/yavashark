use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ThisExpr;
use crate::Error;

impl Context {
    pub fn run_this(&mut self, stmt: &ThisExpr, scope: &mut Scope) -> RuntimeResult {
        let this = scope.this.copy();
        println!("this: {}", scope.this);
        Ok(scope.this.copy())
    }
}
