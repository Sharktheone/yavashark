use swc_ecma_ast::SeqExpr;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter {
    pub fn run_seq(ctx: &mut Context, stmt: &SeqExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
