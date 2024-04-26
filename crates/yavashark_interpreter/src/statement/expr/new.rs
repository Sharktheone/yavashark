use swc_ecma_ast::NewExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_new(&mut self, stmt: &NewExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}