use swc_ecma_ast::ThisExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_this(&mut self, stmt: &ThisExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}