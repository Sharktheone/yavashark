use swc_ecma_ast::{ForHead, ForInStmt};

use yavashark_value::Obj;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use crate::{ControlFlow, Error};

impl Context {
    pub fn run_for_in(&mut self, stmt: &ForInStmt, scope: &mut Scope) -> RuntimeResult {
        let obj = self.run_expr(&stmt.right, stmt.span, scope)?;

        match obj {
            Value::Object(obj) => self.run_for_in_obj(&***obj.get()?, stmt, scope),
            _ => Err(Error::ty_error(format!("{obj:?} is not an object")).into()),
        }
    }

    pub fn run_for_in_obj(
        &mut self,
        obj: &dyn Obj<Self>,
        stmt: &ForInStmt,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let scope = &mut Scope::with_parent(scope)?;
        let label = scope.last_label()?;
        scope.state_set_loop()?;

        let ForHead::VarDecl(v) = &stmt.left else {
            todo!("ForInStmt left is not VarDecl");
        };

        if v.decls.is_empty() {
            ControlFlow::error_syntax("ForInStmt left is empty");
        }

        if v.decls.len() > 1 {
            ControlFlow::error_syntax(
                "Invalid left-hand side in for-in loop: Must have a single binding.",
            );
        }

        let decl = v.decls[0]
            .name
            .clone()
            .ident()
            .ok_or_else(|| ControlFlow::error_syntax("ForInStmt left is not an identifier"))?
            .sym
            .to_string();

        for key in obj.keys() {
            scope.declare_var(decl.clone(), key);

            let result = self.run_statement(&stmt.body, scope);
            match result {
                Ok(_) => {}
                Err(ControlFlow::Return(v)) => return Ok(v),
                Err(ControlFlow::Break(l)) if label.as_ref() == l.as_ref() => {
                    break;
                }
                Err(ControlFlow::Continue(l)) if label.as_ref() == l.as_ref() => {
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
        Ok(Value::Undefined)
    }
}
