use swc_ecma_ast::{BlockStmt, WithStmt};
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_with(&mut self, stmt: &WithStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
