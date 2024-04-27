use swc_ecma_ast::AwaitExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_await(&mut self, stmt: &AwaitExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}