use crate::context::Context;
use crate::scope::Scope;
use crate::{ControlFlow, RuntimeResult};
use crate::Value;
use swc_ecma_ast::ReturnStmt;
use yavashark_value::error::Error;

impl Context {
    pub fn run_return(&mut self, stmt: &ReturnStmt, scope: &mut Scope) -> RuntimeResult {
        let value = if let Some(arg) = &stmt.arg {
            self.run_expr(arg, stmt.span, scope)?
        } else {
            Value::Undefined
        };

        Err(ControlFlow::Return(value))
    }
}
