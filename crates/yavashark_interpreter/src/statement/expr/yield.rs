use swc_ecma_ast::YieldExpr;

use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;

use crate::Interpreter;

impl Interpreter {
    pub fn run_yield(ctx: &mut Context, stmt: &YieldExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
