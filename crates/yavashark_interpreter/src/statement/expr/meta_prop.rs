use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::MetaPropExpr;

impl Context {
    pub fn run_meta_prop(&mut self, stmt: &MetaPropExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
