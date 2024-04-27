use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::NewExpr;
use yavashark_value::error::Error;

impl Context {
    pub fn run_new(&mut self, stmt: &NewExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
