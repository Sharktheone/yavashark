use crate::Interpreter;
use swc_ecma_ast::WithStmt;
use yavashark_env::{Realm, RuntimeResult};

use crate::scope::Scope;

impl Interpreter {
    pub fn run_with(realm: &mut Realm, stmt: &WithStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
