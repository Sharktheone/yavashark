use crate::Interpreter;
use swc_ecma_ast::CondExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_cond(realm: &mut Realm, stmt: &CondExpr, scope: &mut Scope) -> RuntimeResult {
        let test = Self::run_expr(ctx, &stmt.test, stmt.span, scope)?;

        if test.is_truthy() {
            Self::run_expr(ctx, &stmt.cons, stmt.span, scope)
        } else {
            Self::run_expr(ctx, &stmt.alt, stmt.span, scope)
        }
    }
}
