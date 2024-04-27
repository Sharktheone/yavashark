use swc_ecma_ast::BlockStmt;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_block(&mut self, stmt: &BlockStmt, scope: &mut Scope) -> RuntimeResult {
        let scope = &mut Scope::with_parent(scope);
        
        self.run_statements(&stmt.stmts, scope)
    }
}