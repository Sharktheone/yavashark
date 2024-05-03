use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::DoWhileStmt;

impl Context {
    pub fn run_do_while(&mut self, stmt: &DoWhileStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
