use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::Tpl;

impl Context {
    pub fn run_tpl(&mut self, stmt: &Tpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
