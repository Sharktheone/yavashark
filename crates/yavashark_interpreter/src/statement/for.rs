use crate::Value;
use swc_ecma_ast::{ForStmt, VarDeclOrExpr};

use crate::context::Context;
use crate::scope::Scope;
use crate::{ControlFlow, RuntimeResult};

impl Context {
    pub fn run_for(&mut self, stmt: &ForStmt, scope: &mut Scope) -> RuntimeResult {
        let scope = &mut Scope::with_parent(scope);
        let label = scope.last_label();
        scope.state_set_loop();

        if let Some(init) = &stmt.init {
            match init {
                VarDeclOrExpr::VarDecl(v) => {
                    self.decl_var(v, scope)?;
                }
                VarDeclOrExpr::Expr(e) => {
                    self.run_expr(e, stmt.span, scope)?;
                }
            }
        }

        loop {
            if let Some(test) = &stmt.test {
                let value = self.run_expr(test, stmt.span, scope)?;
                if value.is_falsey() {
                    break Ok(Value::Undefined);
                }
            }

            if let Err(e) = self.run_statement(&stmt.body, scope) {
                match e {
                    ControlFlow::Break(l) => {
                        if label.as_ref() == l.as_ref() {
                            break Ok(Value::Undefined);
                        } else {
                            return Err(ControlFlow::Break(l));
                        }
                    }
                    ControlFlow::Continue(l) => {
                        if label.as_ref() == l.as_ref() {
                            continue;
                        } else {
                            return Err(ControlFlow::Continue(l));
                        }
                    }
                    (c) => return Err(c),
                }
            }

            if let Some(update) = &stmt.update {
                self.run_expr(update, stmt.span, scope)?;
            }
        }
    }
}
