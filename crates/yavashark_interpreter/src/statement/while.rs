use swc_ecma_ast::WhileStmt;

use crate::context::Context;
use crate::scope::Scope;
use crate::Value;
use crate::{ControlFlow, RuntimeResult};

impl Context {
    pub fn run_while(&mut self, stmt: &WhileStmt, scope: &mut Scope) -> RuntimeResult {
        let label = scope.last_label();

        loop {
            let cond = self.run_expr(&stmt.test, stmt.span, scope)?;

            if !cond.is_truthy() {
                break Ok(Value::Undefined);
            }

            let result = self.run_statement(&stmt.body, scope);

            match result {
                Ok(_) => {}
                Err(e) => match &e {
                    ControlFlow::Break(l) => {
                        if *l == label {
                            break Ok(Value::Undefined);
                        } else {
                            return Err(e);
                        }
                    }
                    ControlFlow::Continue(l) => {
                        if *l == label {
                            continue;
                        } else {
                            return Err(e);
                        }
                    }
                    _ => return Err(e),
                },
            }
        }
    }
}
