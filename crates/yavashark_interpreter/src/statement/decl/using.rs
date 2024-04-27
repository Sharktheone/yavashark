use swc_ecma_ast::UsingDecl;

use crate::context::Context;
use crate::Res;
use crate::scope::Scope;

impl Context {
    pub fn decl_using(&mut self, stmt: &UsingDecl, scope: &mut Scope) -> Res {
        todo!()
    }
}