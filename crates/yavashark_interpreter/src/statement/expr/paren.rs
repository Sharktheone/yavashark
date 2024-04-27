use swc_ecma_ast::ParenExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_paren(&mut self, stmt: &ParenExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}