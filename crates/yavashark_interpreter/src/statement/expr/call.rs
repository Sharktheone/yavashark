use swc_common::Span;
use swc_ecma_ast::{CallExpr, Callee, ExprOrSpread};

use yavashark_env::scope::Scope;
use yavashark_env::{Context, ControlFlow, Error, Value, ValueResult};

use crate::Interpreter;

impl Interpreter {
    pub fn run_call(ctx: &mut Context, stmt: &CallExpr, scope: &mut Scope) -> ValueResult {
        let Callee::Expr(callee_expr) = &stmt.callee else {
            return Err(Error::ty_error("Unsupported callee".to_string()));
        };

        let callee = Self::run_expr(ctx, callee_expr, stmt.span, scope)?;
        
        Self::run_call_on(ctx, callee, stmt.args.clone(), stmt.span, scope, format!("{:?}", stmt.callee))
    }
    
    
    pub fn run_call_on(ctx: &mut Context, callee: Value, args: Vec<ExprOrSpread>, span: Span, scope: &mut Scope, name: String) -> ValueResult {
        if let Value::Object(f) = callee {
            let args = args
                .iter()
                .map(|arg| Self::run_expr(ctx, &arg.expr, arg.spread.unwrap_or(span), scope))
                .collect::<Result<Vec<Value>, ControlFlow>>()?;

            f.call(ctx, args, scope.this()?.copy()) //In strict mode, this is undefined
        } else {
            Err(Error::ty_error(format!(
                "{name} is not a function",
            )))
        }
    }
}
