use crate::Interpreter;
use swc_ecma_ast::AwaitExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_await(realm: &mut Realm, stmt: &AwaitExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
