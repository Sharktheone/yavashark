use swc_ecma_ast::{Callee, CallExpr};

use yavashark_env::{Context, ControlFlow, Error, Value, ValueResult};
use yavashark_env::scope::Scope;

use crate::Interpreter;

impl Interpreter {
    pub fn run_call(ctx: &mut Context, stmt: &CallExpr, scope: &mut Scope) -> ValueResult {
        let Callee::Expr(callee_expr) = &stmt.callee else {
            return Err(Error::ty_error("Unsupported callee".to_string()));
        };

        let callee = Self::run_expr(ctx, callee_expr, stmt.span, scope)?;

        if let Value::Object(f) = callee {
            let args = stmt
                .args
                .iter()
                .map(|arg| Self::run_expr(ctx, &arg.expr, arg.spread.unwrap_or(stmt.span), scope))
                .collect::<Result<Vec<Value>, ControlFlow>>()?;

            f.call(ctx, args, scope.this()?.copy()) //In strict mode, this is undefined
        } else {
            Err(Error::ty_error(format!(
                "{:?} ia not a function",
                stmt.callee
            )))
        }
    }
}
