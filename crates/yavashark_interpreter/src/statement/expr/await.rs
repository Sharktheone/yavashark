use swc_ecma_ast::AwaitExpr;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_await(&mut self, stmt: &AwaitExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
