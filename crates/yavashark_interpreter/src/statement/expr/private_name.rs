use swc_ecma_ast::PrivateName;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_private_name(&mut self, stmt: &PrivateName, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}