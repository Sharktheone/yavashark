use crate::context::Context;
use crate::scope::Scope;
use crate::Value;
use crate::{ControlFlow, RuntimeResult};
use swc_ecma_ast::Ident;
use yavashark_value::error::Error;

impl Context {
    pub fn run_ident(&mut self, stmt: &Ident, scope: &mut Scope) -> RuntimeResult {
        let ident = stmt.sym.to_string();
        let value = scope.resolve(&ident);
        match value {
            Some(value) => Ok(value.copy()),
            None => Err(ControlFlow::error_reference(format!(
                "{} is not defined",
                ident
            ))),
        }
    }
}
