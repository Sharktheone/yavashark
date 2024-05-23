use swc_ecma_ast::MetaPropExpr;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_meta_prop(&mut self, stmt: &MetaPropExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
