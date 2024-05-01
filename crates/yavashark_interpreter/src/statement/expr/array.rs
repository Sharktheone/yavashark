use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ArrayLit;
use crate::Error;

impl Context {
    pub fn run_array(&mut self, stmt: &ArrayLit, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
