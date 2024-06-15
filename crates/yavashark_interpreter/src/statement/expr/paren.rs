use swc_ecma_ast::ParenExpr;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl  Interpreter{
    pub fn run_paren(ctx: &mut Context, stmt: &ParenExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
