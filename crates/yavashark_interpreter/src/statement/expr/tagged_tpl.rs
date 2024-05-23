use swc_ecma_ast::TaggedTpl;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_tagged_tpl(&mut self, stmt: &TaggedTpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
