use crate::Interpreter;
use swc_ecma_ast::ThrowStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult};

impl Interpreter {
    pub fn run_throw(realm: &mut Realm, stmt: &ThrowStmt, scope: &mut Scope) -> RuntimeResult {
        Err(ControlFlow::throw(Self::run_expr(
            realm, &stmt.arg, stmt.span, scope,
        )?))
    }
}
