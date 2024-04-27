use swc_ecma_ast::CallExpr;
use yavashark_value::error::Error;
use crate::{Value, ValueResult};
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_call(&mut self, stmt: &CallExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}