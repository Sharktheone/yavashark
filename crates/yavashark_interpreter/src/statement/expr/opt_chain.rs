use crate::Interpreter;
use swc_ecma_ast::OptChainExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_opt_chain(
        ctx: &mut Context,
        stmt: &OptChainExpr,
        scope: &mut Scope,
    ) -> RuntimeResult {
        todo!()
    }
}
