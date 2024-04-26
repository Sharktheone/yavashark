use swc_ecma_ast::TaggedTpl;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_tagged_tpl(&mut self, stmt: &TaggedTpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}