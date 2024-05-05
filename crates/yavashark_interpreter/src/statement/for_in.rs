use swc_ecma_ast::{ForHead, ForInStmt};

use yavashark_value::Obj;

use crate::{ControlFlow, Error};
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;
use crate::Value;

impl Context {
    pub fn run_for_in(&mut self, stmt: &ForInStmt, scope: &mut Scope) -> RuntimeResult {
        let obj = self.run_expr(&stmt.right, stmt.span, scope)?;


        match obj {
            Value::Object(obj) => {
                self.run_for_in_obj(&**obj.get()?, stmt, scope)
            }
            Value::Function(func) => {
                self.run_for_in_obj(func.get()?.as_object(), stmt, scope)
            }
            _ => {
                Err(Error::ty(format!("{:?} is not an object", obj)).into())
            }
        }
    }

    pub fn run_for_in_obj(&mut self, obj: &dyn Obj<Context>, stmt: &ForInStmt, scope: &mut Scope) -> RuntimeResult {
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

        for key in obj.keys() {
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
                }
                Err(ControlFlow::Continue(l)) => {
                    if label.as_ref() == l.as_ref() {
                        continue;
                    } else {
                        return Err(ControlFlow::Continue(l));
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(Value::Undefined)
    }
}
