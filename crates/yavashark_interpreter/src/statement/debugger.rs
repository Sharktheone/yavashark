use swc_ecma_ast::DebuggerStmt;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_debugger(&mut self, stmt: &DebuggerStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
