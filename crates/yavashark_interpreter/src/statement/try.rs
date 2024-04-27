use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::TryStmt;
use yavashark_value::error::Error;

impl Context {
    pub fn run_try(&mut self, stmt: &TryStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
