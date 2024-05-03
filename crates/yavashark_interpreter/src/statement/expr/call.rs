use swc_ecma_ast::{CallExpr, Callee};

use crate::context::Context;
use crate::scope::Scope;
use crate::ControlFlow;
use crate::ValueResult;
use crate::{Error, Value};

impl Context {
    pub fn run_call(&mut self, stmt: &CallExpr, scope: &mut Scope) -> ValueResult {
        let Callee::Expr(callee_expr) = &stmt.callee else {
            return Err(Error::ty("Unsupported callee".to_string()));
        };

        let callee = self.run_expr(callee_expr, stmt.span, scope)?;

        if let Value::Function(f) = callee {
            let args = stmt
                .args
                .iter()
                .map(|arg| self.run_expr(&arg.expr, arg.spread.unwrap_or(stmt.span), scope))
                .collect::<Result<Vec<Value>, ControlFlow>>()?;

            f.call(self, args, scope.this.copy()) //In strict mode, this is undefined
        } else {
            Err(Error::ty(format!("{:?} ia not a function", stmt.callee)))
        }
    }
}
