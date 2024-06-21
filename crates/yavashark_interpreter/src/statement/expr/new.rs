use swc_ecma_ast::NewExpr;

use yavashark_env::scope::Scope;
use yavashark_env::{Context, ControlFlow, Object, RuntimeResult, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn run_new(ctx: &mut Context, stmt: &NewExpr, scope: &mut Scope) -> RuntimeResult {
        let callee = Self::run_expr(ctx, &stmt.callee, stmt.span, scope)?;

        if let Value::Object(f) = callee {
            let mut call_args = Vec::with_capacity(0);

            if let Some(args) = &stmt.args {
                call_args.reserve(args.len());

                for arg in args {
                    call_args.push(Self::run_expr(
                        ctx,
                        &arg.expr,
                        arg.spread.unwrap_or(stmt.span),
                        scope,
                    )?);
                    if arg.spread.is_some() {
                        todo!("spread")
                    }
                }
            }

            let this = f.get_constructor_value(ctx).unwrap_or(Object::new(ctx).into());

            let _ = f.call(ctx, call_args, this.copy())?;

            Ok(this) //This is always an object, so it will also be updated when we copy it
        } else {
            Err(ControlFlow::error_type(format!(
                "{:?} ia not a constructor",
                stmt.callee
            )))
        }
    }
}
