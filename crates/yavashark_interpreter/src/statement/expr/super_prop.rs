use swc_ecma_ast::SuperPropExpr;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl  Interpreter{
    pub fn run_super_prop(ctx: &mut Context, stmt: &SuperPropExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
