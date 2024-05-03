use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ParenExpr;

impl Context {
    pub fn run_paren(&mut self, stmt: &ParenExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
