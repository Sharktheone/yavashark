use swc_ecma_ast::Ident;

use crate::context::Context;
use crate::scope::Scope;
use crate::{ControlFlow, RuntimeResult};

impl Context {
    pub fn run_ident(&mut self, stmt: &Ident, scope: &mut Scope) -> RuntimeResult {
        let ident = stmt.sym.to_string();
        let value = scope.resolve(&ident)?;
        value.map_or_else(
            || {
                Err(ControlFlow::error_reference(format!(
                    "{ident} is not defined"
                )))
            },
            Ok,
        )
    }
}
