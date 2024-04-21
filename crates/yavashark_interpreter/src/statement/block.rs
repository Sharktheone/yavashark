use swc_ecma_ast::BlockStmt;
use yavashark_value::error::Error;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_block(&mut self, stmt: &BlockStmt, scope: &mut Scope) -> Result<(), Error> {
        todo!()
    }
}