use swc_ecma_ast::UsingDecl;

use crate::context::Context;
use crate::scope::Scope;
use crate::Res;

impl Context {
    pub fn decl_using(&mut self, stmt: &UsingDecl, scope: &mut Scope) -> Res {
        todo!()
    }
}
