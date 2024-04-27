use swc_ecma_ast::FnExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_fn(&mut self, stmt: &FnExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}