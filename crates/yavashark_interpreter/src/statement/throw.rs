use swc_ecma_ast::ThrowStmt;

use yavashark_value::error::Error;
use crate::Value;

use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_throw(&mut self, stmt: &ThrowStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
