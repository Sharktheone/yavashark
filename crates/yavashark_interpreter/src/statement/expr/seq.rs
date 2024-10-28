use crate::Interpreter;
use swc_ecma_ast::SeqExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_seq(realm: &mut Realm, stmt: &SeqExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
