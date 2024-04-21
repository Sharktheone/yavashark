use swc_ecma_ast::ContinueStmt;
use yavashark_value::error::Error;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_continue(&mut self, stmt: &ContinueStmt, scope: &mut Scope) -> Result<(), Error> {
        todo!()
    }
}
