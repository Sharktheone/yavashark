use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::BlockStmt;

impl Context {
    pub fn run_block(&mut self, stmt: &BlockStmt, scope: &mut Scope) -> RuntimeResult {
        let scope = &mut Scope::with_parent(scope);

        self.run_statements(&stmt.stmts, scope)
    }
    pub fn run_block_this(
        &mut self,
        stmt: &BlockStmt,
        scope: &mut Scope,
        this: Value,
    ) -> RuntimeResult {
        let scope = &mut Scope::with_parent_this(scope, this);

        self.run_statements(&stmt.stmts, scope)
    }
}
