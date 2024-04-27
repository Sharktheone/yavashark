use swc_ecma_ast::Tpl;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_tpl(&mut self, stmt: &Tpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}