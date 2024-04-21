use swc_ecma_ast::TryStmt;
use yavashark_value::error::Error;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_try(&mut self, stmt: &TryStmt, scope: &mut Scope) -> Result<(), Error> {
        todo!()
    }
}
