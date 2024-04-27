use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ForStmt;
use yavashark_value::error::Error;

impl Context {
    pub fn run_for(&mut self, stmt: &ForStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
