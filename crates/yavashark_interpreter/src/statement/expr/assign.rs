use swc_ecma_ast::AssignExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_assign(&mut self, stmt: &AssignExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}