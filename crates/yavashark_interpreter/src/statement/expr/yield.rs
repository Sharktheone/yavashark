use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::YieldExpr;

impl Context {
    pub fn run_yield(&mut self, stmt: &YieldExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
