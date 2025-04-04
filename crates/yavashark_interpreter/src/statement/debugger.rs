use crate::Interpreter;
use swc_ecma_ast::DebuggerStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_debugger(
        realm: &mut Realm,
        stmt: &DebuggerStmt,
        scope: &mut Scope,
    ) -> RuntimeResult {
        println!("Set breakpoint here");
        Ok(Value::Undefined)
    }
}
