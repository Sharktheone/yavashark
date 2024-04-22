use swc_ecma_ast::BreakStmt;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_break(&mut self, stmt: &BreakStmt, scope: &mut Scope) -> Result<Value, Error> {
        todo!()
    }
}
