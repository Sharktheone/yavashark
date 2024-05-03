use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::FnExpr;

impl Context {
    pub fn run_fn(&mut self, stmt: &FnExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
