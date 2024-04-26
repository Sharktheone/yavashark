use swc_ecma_ast::LabeledStmt;

use yavashark_value::error::Error;
use yavashark_value::Value;

use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_labeled(&mut self, stmt: &LabeledStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
