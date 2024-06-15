use swc_ecma_ast::DebuggerStmt;
use yavashark_env::{Context, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;

impl Interpreter {
    pub fn run_debugger(ctx: &mut Context, stmt: &DebuggerStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
