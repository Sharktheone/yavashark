use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ForOfStmt;

impl Context {
    pub fn run_for_of(&mut self, stmt: &ForOfStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
