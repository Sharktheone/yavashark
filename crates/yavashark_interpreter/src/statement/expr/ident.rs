use swc_ecma_ast::Ident;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::{ControlFlow, RuntimeResult};
use crate::scope::Scope;

impl Context {
    pub fn run_ident(&mut self, stmt: &Ident, scope: &mut Scope) -> RuntimeResult {
        let ident = stmt.sym.to_string();
        let value = scope.resolve(&ident);
        match value {
            Some(value) => Ok(value.clone()),
            None => Err(ControlFlow::error_reference(format!("{} is not defined", ident)))
        }
    }
}