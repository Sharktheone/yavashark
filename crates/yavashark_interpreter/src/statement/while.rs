use swc_ecma_ast::WhileStmt;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_while(&mut self, stmt: &WhileStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
