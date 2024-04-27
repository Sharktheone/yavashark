use swc_ecma_ast::YieldExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_yield(&mut self, stmt: &YieldExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}