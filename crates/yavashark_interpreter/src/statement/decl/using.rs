use swc_ecma_ast::UsingDecl;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn decl_using(&mut self, stmt: &UsingDecl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}