use crate::Interpreter;
use swc_ecma_ast::ThisExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_this(realm: &mut Realm, stmt: &ThisExpr, scope: &mut Scope) -> RuntimeResult {
        Ok(scope.this()?.copy())
    }
}
