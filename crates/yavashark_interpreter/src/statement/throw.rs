use swc_ecma_ast::ThrowStmt;

use crate::context::Context;
use crate::ControlFlow;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_throw(&mut self, stmt: &ThrowStmt, scope: &mut Scope) -> RuntimeResult {
        Err(ControlFlow::throw(self.run_expr(&stmt.arg, stmt.span, scope)?))
    }
}
