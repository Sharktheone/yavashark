use swc_ecma_ast::DoWhileStmt;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_do_while(&mut self, stmt: &DoWhileStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
