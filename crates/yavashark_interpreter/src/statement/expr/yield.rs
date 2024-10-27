use swc_ecma_ast::YieldExpr;

use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

use crate::Interpreter;

impl Interpreter {
    pub fn run_yield(realm: &mut Realm, stmt: &YieldExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
