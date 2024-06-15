use swc_ecma_ast::MetaPropExpr;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter {
    pub fn run_meta_prop(ctx: &mut Context, stmt: &MetaPropExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
