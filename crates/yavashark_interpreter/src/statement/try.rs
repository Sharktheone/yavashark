use swc_ecma_ast::TryStmt;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_try(&mut self, stmt: &TryStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
