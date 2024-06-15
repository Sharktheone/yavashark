use swc_ecma_ast::AwaitExpr;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;

impl Interpreter{
    pub fn run_await(ctx: &mut Context, stmt: &AwaitExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
