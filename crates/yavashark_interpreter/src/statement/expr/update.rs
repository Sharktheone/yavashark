use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::UpdateExpr;
use yavashark_value::error::Error;

impl Context {
    pub fn run_update(&mut self, stmt: &UpdateExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
