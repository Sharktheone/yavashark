use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::{BlockStmt, WithStmt};
use crate::Error;

impl Context {
    pub fn run_with(&mut self, stmt: &WithStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
