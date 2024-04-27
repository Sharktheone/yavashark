use swc_ecma_ast::FnDecl;

use crate::context::Context;
use crate::Res;
use crate::scope::Scope;

impl Context {
    pub fn decl_fn(&mut self, stmt: &FnDecl, scope: &mut Scope) -> Res {
        todo!()
    }
}