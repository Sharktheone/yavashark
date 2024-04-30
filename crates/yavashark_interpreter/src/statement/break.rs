use swc_ecma_ast::BreakStmt;

use crate::{ControlFlow, Value};
use yavashark_value::error::Error;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_break(&mut self, stmt: &BreakStmt, scope: &mut Scope) -> RuntimeResult {
        if let Some(label) = &stmt.label {
            if !scope.has_label(&label.sym.to_string()) {
                return Err(ControlFlow::error_reference(format!("Label {} not found", label.sym)));
            }
        }
        Err(ControlFlow::Break(stmt.label.as_ref().map(|l| l.sym.to_string())))
    }
}
