use crate::Interpreter;
use swc_ecma_ast::CondExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_cond(realm: &mut Realm, stmt: &CondExpr, scope: &mut Scope) -> RuntimeResult {
        let test = Self::run_expr(realm, &stmt.test, stmt.span, scope)?;

        if test.is_truthy() {
            Self::run_expr(realm, &stmt.cons, stmt.span, scope)
        } else {
            Self::run_expr(realm, &stmt.alt, stmt.span, scope)
        }
    }
}
