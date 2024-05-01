use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::DebuggerStmt;
use crate::Error;

impl Context {
    pub fn run_debugger(&mut self, stmt: &DebuggerStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
