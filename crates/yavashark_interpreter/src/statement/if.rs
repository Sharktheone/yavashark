use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::IfStmt;

impl Context {
    pub fn run_if(&mut self, stmt: &IfStmt, scope: &mut Scope) -> RuntimeResult {
        let test = self.run_expr(&stmt.test, stmt.span, scope)?;

        if test.is_truthy() {
            self.run_statement(&stmt.cons, scope)
        } else if let Some(alt) = &stmt.alt {
            self.run_statement(alt, scope)
        } else {
            Ok(Value::Undefined)
        }
    }
}
