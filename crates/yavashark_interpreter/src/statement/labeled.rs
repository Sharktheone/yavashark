use swc_ecma_ast::LabeledStmt;

use crate::Value;
use yavashark_value::error::Error;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_labeled(&mut self, stmt: &LabeledStmt, scope: &mut Scope) -> RuntimeResult {
        scope.declare_label(stmt.label.sym.to_string());
        self.run_statement(&stmt.body, scope)
    }
}
