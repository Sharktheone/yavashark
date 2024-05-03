use crate::Value;
use swc_ecma_ast::SwitchStmt;

use crate::context::Context;
use crate::scope::Scope;
use crate::ControlFlow;
use crate::RuntimeResult;

impl Context {
    pub fn run_switch(&mut self, stmt: &SwitchStmt, scope: &mut Scope) -> RuntimeResult {
        let discriminant = self.run_expr(&stmt.discriminant, stmt.span, scope)?;
        let scope = &mut Scope::with_parent(scope);
        scope.state_set_breakable();

        for case in &stmt.cases {
            if let Some(test) = &case.test {
                let test = self.run_expr(test, case.span, scope)?;
                if discriminant == test {
                } else {
                    continue;
                }
            }

            if let Err(e) = self.run_statements(&case.cons, scope) {
                return match &e {
                    ControlFlow::Break(_) => Ok(Value::Undefined),
                    _ => Err(e),
                };
            }
        }

        Ok(Value::Undefined)
    }
}
