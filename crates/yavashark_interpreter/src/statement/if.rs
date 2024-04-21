use swc_ecma_ast::IfStmt;
use yavashark_value::error::Error;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_if(&mut self, stmt: &IfStmt, scope: &mut Scope) -> Result<(), Error> {
        todo!()
    }
}
