use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ForInStmt;

impl Context {
    pub fn run_for_in(&mut self, stmt: &ForInStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
