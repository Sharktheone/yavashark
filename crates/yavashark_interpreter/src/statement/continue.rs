use crate::context::Context;
use crate::scope::Scope;
use crate::{ControlFlow, RuntimeResult};
use crate::Value;
use swc_ecma_ast::ContinueStmt;
use yavashark_value::error::Error;

impl Context {
    pub fn run_continue(&mut self, stmt: &ContinueStmt, scope: &mut Scope) -> RuntimeResult {
        if let Some(label) = &stmt.label {
            if !scope.has_label(&label.sym.to_string()) {
                return Err(ControlFlow::error_reference(format!("Label {} not found", label.sym)));
            }
        }
        Err(ControlFlow::Continue(stmt.label.as_ref().map(|l| l.sym.to_string())))
    }
}
