use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::DoWhileStmt;

impl Context {
    pub fn run_do_while(&mut self, stmt: &DoWhileStmt, scope: &mut Scope) -> RuntimeResult {
        let mut result = Value::Undefined;

        loop {
            result = self.run_statement(&stmt.body, scope)?;
            
            let condition = self.run_expr(&stmt.test, stmt.span, scope)?;

            if condition.is_falsey() {
                break;
            }
        }

        Ok(result)
    }
}
