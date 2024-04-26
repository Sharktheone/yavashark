use swc_ecma_ast::Ident;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_ident(&mut self, stmt: &Ident, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}