use swc_ecma_ast::ThrowStmt;

use crate::Value;
use yavashark_value::error::Error;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_throw(&mut self, stmt: &ThrowStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
