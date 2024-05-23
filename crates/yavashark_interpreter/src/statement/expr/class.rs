use swc_ecma_ast::ClassExpr;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_class(&mut self, stmt: &ClassExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
