use swc_ecma_ast::Tpl;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_tpl(&mut self, stmt: &Tpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
