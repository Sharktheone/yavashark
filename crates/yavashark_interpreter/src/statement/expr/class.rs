use swc_ecma_ast::ClassExpr;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter {
    pub fn run_class(ctx: &mut Context, stmt: &ClassExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
