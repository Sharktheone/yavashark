use swc_ecma_ast::ThrowStmt;
use yavashark_env::{Context, ControlFlow, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter {
    pub fn run_throw(ctx: &mut Context, stmt: &ThrowStmt, scope: &mut Scope) -> RuntimeResult {
        Err(ControlFlow::throw(
            Self::run_expr(ctx, &stmt.arg, stmt.span, scope)?,
        ))
    }
}
