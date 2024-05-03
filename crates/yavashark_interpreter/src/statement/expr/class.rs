use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ClassExpr;

impl Context {
    pub fn run_class(&mut self, stmt: &ClassExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
