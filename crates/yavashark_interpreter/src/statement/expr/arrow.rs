use swc_ecma_ast::ArrowExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_arrow(&mut self, stmt: &ArrowExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}