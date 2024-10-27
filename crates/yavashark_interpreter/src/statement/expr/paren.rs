use crate::Interpreter;
use swc_ecma_ast::ParenExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_paren(realm: &mut Realm, stmt: &ParenExpr, scope: &mut Scope) -> RuntimeResult {
        Self::run_expr(ctx, &stmt.expr, stmt.span, scope)
    }
}
