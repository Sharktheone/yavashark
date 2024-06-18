use crate::Interpreter;
use swc_ecma_ast::ThisExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_this(ctx: &mut Context, stmt: &ThisExpr, scope: &mut Scope) -> RuntimeResult {
        Ok(scope.this()?.copy())
    }
}
