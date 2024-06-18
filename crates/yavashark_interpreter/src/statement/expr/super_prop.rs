use crate::Interpreter;
use swc_ecma_ast::SuperPropExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_super_prop(
        ctx: &mut Context,
        stmt: &SuperPropExpr,
        scope: &mut Scope,
    ) -> RuntimeResult {
        todo!()
    }
}
