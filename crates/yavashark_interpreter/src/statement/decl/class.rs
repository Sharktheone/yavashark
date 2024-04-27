use swc_ecma_ast::ClassDecl;

use crate::context::Context;
use crate::scope::Scope;
use crate::Res;

impl Context {
    pub fn decl_class(&mut self, stmt: &ClassDecl, scope: &mut Scope) -> Res {
        todo!()
    }
}
