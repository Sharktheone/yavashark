use swc_ecma_ast::ArrayLit;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_array(&mut self, stmt: &ArrayLit, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}