use swc_ecma_ast::BlockStmt;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_block(&mut self, stmt: &BlockStmt, scope: &mut Scope) -> Result<Value, Error> {
        let scope = &mut Scope::with_parent(scope);
        
        self.run_statements(&stmt.stmts, scope)
    }
}