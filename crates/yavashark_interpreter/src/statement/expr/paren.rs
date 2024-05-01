use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ParenExpr;
use crate::Error;

impl Context {
    pub fn run_paren(&mut self, stmt: &ParenExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
