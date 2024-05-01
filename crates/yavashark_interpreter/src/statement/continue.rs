use crate::context::Context;
use crate::scope::Scope;
use crate::{ControlFlow, RuntimeResult};
use crate::Value;
use swc_ecma_ast::ContinueStmt;
use crate::Error;

impl Context {
    pub fn run_continue(&mut self, stmt: &ContinueStmt, scope: &mut Scope) -> RuntimeResult {
        if !scope.state_is_continuable() {
            return Err(ControlFlow::error_syntax("Illegal continue statement"));
        }
        
        if let Some(label) = &stmt.label {
            if !scope.has_label(label.sym.as_ref()) {
                return Err(ControlFlow::error_reference(format!("Label {} not found", label.sym)));
            }
        }
        Err(ControlFlow::Continue(stmt.label.as_ref().map(|l| l.sym.to_string())))
    }
}
