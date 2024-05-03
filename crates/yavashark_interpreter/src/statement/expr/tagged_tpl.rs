use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::TaggedTpl;

impl Context {
    pub fn run_tagged_tpl(&mut self, stmt: &TaggedTpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
