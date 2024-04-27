use swc_ecma_ast::ForOfStmt;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_for_of(&mut self, stmt: &ForOfStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
