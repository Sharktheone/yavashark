use crate::Interpreter;
use swc_ecma_ast::CondExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_cond(ctx: &mut Context, stmt: &CondExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
