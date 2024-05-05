use crate::context::Context;
use crate::scope::Scope;
use crate::{ControlFlow, Error};
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::{ForHead, ForOfStmt};

impl Context {
    pub fn run_for_of(&mut self, stmt: &ForOfStmt, scope: &mut Scope) -> RuntimeResult {
        let obj = self.run_expr(&stmt.right, stmt.span, scope)?;


        if let Value::Object(obj) = obj {
            let scope = &mut Scope::with_parent(scope);
            let label = scope.last_label();
            scope.state_set_loop();

            let ForHead::VarDecl(v) = &stmt.left else {
                todo!("ForInStmt left is not VarDecl");
            };

            if v.decls.is_empty() {
                ControlFlow::error_syntax("ForInStmt left is empty");
            }

            if v.decls.len() > 1 {
                ControlFlow::error_syntax("Invalid left-hand side in for-in loop: Must have a single binding.");
            }

            let decl = v.decls[0].name.clone()
                .ident().ok_or_else(|| ControlFlow::error_syntax("ForInStmt left is not an identifier"))?
                .sym.to_string();

            for key in obj.values()? {
                scope.declare_var(decl.clone(), key);

                let result = self.run_statement(&stmt.body, scope);
                match result {
                    Ok(_) => {}
                    Err(ControlFlow::Return(v)) => return Ok(v),
                    Err(ControlFlow::Break(l)) => {
                        if label.as_ref() == l.as_ref() {
                            break;
                        } else {
                            return Err(ControlFlow::Break(l));
                        }
                    },
                    Err(ControlFlow::Continue(l)) => {
                        if label.as_ref() == l.as_ref() {
                            continue;
                        } else {
                            return Err(ControlFlow::Continue(l));
                        }
                    },
                    Err(e) => return Err(e),
                }
            }
        } else if let Value::Function(func) = obj {} else {
            return Err(Error::ty(format!("{:?} is not an object", obj)).into());
        }

        Ok(Value::Undefined)
    }
}
