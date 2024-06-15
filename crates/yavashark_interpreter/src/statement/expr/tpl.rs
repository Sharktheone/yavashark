use swc_ecma_ast::Tpl;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter{
    pub fn run_tpl(ctx: &mut Context, stmt: &Tpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
