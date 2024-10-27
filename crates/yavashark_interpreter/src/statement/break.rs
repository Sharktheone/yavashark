use swc_ecma_ast::BreakStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, ControlFlow, RuntimeResult};

use crate::Interpreter;

impl Interpreter {
    pub fn run_break(realm: &mut Realm, stmt: &BreakStmt, scope: &mut Scope) -> RuntimeResult {
        if !scope.state_is_breakable()? {
            return Err(ControlFlow::error_syntax("Illegal break statement"));
        }

        if let Some(label) = &stmt.label {
            if !scope.has_label(label.sym.as_ref())? {
                return Err(ControlFlow::error_reference(format!(
                    "Label {} not found",
                    label.sym
                )));
            }
        }
        Err(ControlFlow::Break(
            stmt.label.as_ref().map(|l| l.sym.to_string()),
        ))
    }
}
