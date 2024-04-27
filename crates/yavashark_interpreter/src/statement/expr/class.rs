use swc_ecma_ast::ClassExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_class(&mut self, stmt: &ClassExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}