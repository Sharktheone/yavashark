use crate::Interpreter;
use swc_ecma_ast::LabeledStmt;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_labeled(ctx: &mut Context, stmt: &LabeledStmt, scope: &mut Scope) -> RuntimeResult {
        scope.declare_label(stmt.label.sym.to_string());
        Self::run_statement(ctx, &stmt.body, scope)
    }
}
