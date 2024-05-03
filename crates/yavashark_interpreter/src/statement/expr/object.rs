use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ObjectLit;

impl Context {
    pub fn run_object(&mut self, stmt: &ObjectLit, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
