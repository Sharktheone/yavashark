use swc_ecma_ast::BreakStmt;
use yavashark_value::error::Error;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_break(&mut self, stmt: &BreakStmt, scope: &mut Scope) -> Result<(), Error> {
        todo!()
    }
}
