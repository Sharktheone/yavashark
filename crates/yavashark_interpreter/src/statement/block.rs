use swc_ecma_ast::BlockStmt;

use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn run_block(ctx: &mut Context, stmt: &BlockStmt, scope: &mut Scope) -> RuntimeResult {
        let scope = &mut Scope::with_parent(scope)?;

        Self::run_statements(ctx, &stmt.stmts, scope)
    }
    pub fn run_block_this(
        ctx: &mut Context,
        stmt: &BlockStmt,
        scope: &mut Scope,
        this: Value,
    ) -> RuntimeResult {
        let scope = &mut Scope::with_parent_this(scope, this)?;

        Self::run_statements(ctx, &stmt.stmts, scope)
    }
}
