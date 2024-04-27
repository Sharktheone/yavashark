use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::AwaitExpr;
use yavashark_value::error::Error;

impl Context {
    pub fn run_await(&mut self, stmt: &AwaitExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
