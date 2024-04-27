use swc_ecma_ast::BreakStmt;

use crate::Value;
use yavashark_value::error::Error;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_break(&mut self, stmt: &BreakStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
