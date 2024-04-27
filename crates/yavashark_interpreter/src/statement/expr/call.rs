use crate::context::Context;
use crate::scope::Scope;
use crate::{ControlFlow, RuntimeResult};
use crate::{Value, ValueResult};
use swc_ecma_ast::{CallExpr, Callee};
use yavashark_value::error::Error;

impl Context {
    pub fn run_call(&mut self, stmt: &CallExpr, scope: &mut Scope) -> ValueResult {
        let Callee::Expr(callee_expr) = &stmt.callee else {
            return Err(Error::ty("Unsupported callee".to_string()));
        };

        let callee = self.run_expr(callee_expr, stmt.span, scope)?;

        return if let Value::Object(obj) = callee {
            let mut obj = obj
                .try_borrow_mut()
                .map_err(|_| Error::reference("Cannot borrow object".to_string()))?;

            if let Some(f) = &mut obj.call {
                let args = stmt
                    .args
                    .iter()
                    .map(|arg| self.run_expr(&arg.expr, arg.spread.unwrap_or(stmt.span), scope))
                    .collect::<Result<Vec<Value>, ControlFlow>>()?;

                f.call(args, scope)
            } else {
                Err(Error::ty(format!("{:?} ia not a function", stmt.callee)))
            }
        } else {
            Err(Error::ty(format!("{:?} ia not a function", stmt.callee)))
        };
    }
}
