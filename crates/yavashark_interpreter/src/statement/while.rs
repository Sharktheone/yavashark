use swc_ecma_ast::WhileStmt;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_while(&mut self, stmt: &WhileStmt, scope: &mut Scope) -> Result<Value, Error> {
        todo!()
    }
}