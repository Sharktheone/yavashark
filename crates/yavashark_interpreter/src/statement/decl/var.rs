use swc_ecma_ast::VarDecl;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn decl_var(&mut self, stmt: &VarDecl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}