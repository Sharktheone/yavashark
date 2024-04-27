use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ArrowExpr;
use yavashark_value::error::Error;

impl Context {
    pub fn run_arrow(&mut self, stmt: &ArrowExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
