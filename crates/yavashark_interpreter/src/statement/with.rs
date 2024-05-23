use swc_ecma_ast::WithStmt;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_with(&mut self, stmt: &WithStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
