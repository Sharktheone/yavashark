use crate::Interpreter;
use swc_ecma_ast::Tpl;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_tpl(ctx: &mut Context, stmt: &Tpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
