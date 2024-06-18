use crate::Interpreter;
use swc_ecma_ast::WithStmt;
use yavashark_env::{Context, RuntimeResult};

use crate::scope::Scope;

impl Interpreter {
    pub fn run_with(ctx: &mut Context, stmt: &WithStmt, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
