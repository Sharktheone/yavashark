use swc_ecma_ast::ForStmt;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_for(&mut self, stmt: &ForStmt, scope: &mut Scope) -> Result<Value, Error> {
        todo!()
    }
}
