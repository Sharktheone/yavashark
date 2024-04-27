use swc_ecma_ast::CondExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_cond(&mut self, stmt: &CondExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}