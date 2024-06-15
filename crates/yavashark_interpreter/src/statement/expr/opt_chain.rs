use swc_ecma_ast::OptChainExpr;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter{
    pub fn run_opt_chain(ctx: &mut Context, stmt: &OptChainExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
