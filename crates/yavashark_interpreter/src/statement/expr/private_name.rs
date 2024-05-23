use swc_ecma_ast::PrivateName;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_private_name(&mut self, stmt: &PrivateName, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
