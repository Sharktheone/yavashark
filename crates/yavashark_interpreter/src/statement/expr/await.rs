use crate::Interpreter;
use swc_common::Spanned;
use swc_ecma_ast::AwaitExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_await(realm: &mut Realm, stmt: &AwaitExpr, scope: &mut Scope) -> RuntimeResult {
        Self::run_expr(realm, &stmt.arg, stmt.arg.span(), scope)
    }
}
