use swc_ecma_ast::ForInStmt;
use yavashark_value::error::Error;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_for_in(&mut self, stmt: &ForInStmt, scope: &mut Scope) -> Result<(), Error> {
        todo!()
    }
}
