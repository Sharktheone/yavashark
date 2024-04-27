use swc_ecma_ast::SwitchStmt;

use crate::Value;
use yavashark_value::error::Error;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_switch(&mut self, stmt: &SwitchStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
