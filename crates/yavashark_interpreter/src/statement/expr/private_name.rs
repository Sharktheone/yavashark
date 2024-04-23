use swc_ecma_ast::PrivateName;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_private_name(&mut self, stmt: &PrivateName, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}