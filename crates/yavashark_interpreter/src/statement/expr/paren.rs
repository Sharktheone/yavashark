use crate::Interpreter;
use swc_ecma_ast::ParenExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_paren(realm: &mut Realm, stmt: &ParenExpr, scope: &mut Scope) -> RuntimeResult {
        Self::run_expr(realm, &stmt.expr, stmt.span, scope)
    }
}
