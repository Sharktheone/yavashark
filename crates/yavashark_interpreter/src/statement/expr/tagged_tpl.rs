use swc_ecma_ast::TaggedTpl;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter { 
    pub fn run_tagged_tpl(ctx: &mut Context, stmt: &TaggedTpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
