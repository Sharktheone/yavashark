use swc_ecma_ast::ThisExpr;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter {
    pub fn run_this(ctx: &mut Context, stmt: &ThisExpr, scope: &mut Scope) -> RuntimeResult {
        Ok(scope.this()?.copy())
    }
}
