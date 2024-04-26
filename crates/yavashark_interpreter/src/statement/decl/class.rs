use swc_ecma_ast::ClassDecl;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn decl_class(&mut self, stmt: &ClassDecl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}