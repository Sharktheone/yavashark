use crate::Interpreter;
use swc_ecma_ast::ClassExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_class(realm: &mut Realm, stmt: &ClassExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
