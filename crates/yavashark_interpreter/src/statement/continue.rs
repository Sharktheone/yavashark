use swc_ecma_ast::ContinueStmt;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_continue(&mut self, stmt: &ContinueStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
