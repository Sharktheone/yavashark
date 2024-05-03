use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::CondExpr;

impl Context {
    pub fn run_cond(&mut self, stmt: &CondExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
