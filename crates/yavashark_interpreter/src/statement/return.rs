use swc_ecma_ast::ReturnStmt;
use yavashark_value::error::Error;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_return(&mut self, stmt: &ReturnStmt, scope: &mut Scope) -> Result<(), Error> {
        todo!()
    }
}
