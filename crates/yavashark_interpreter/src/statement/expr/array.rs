use swc_ecma_ast::ArrayLit;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_array(&mut self, stmt: &ArrayLit, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}