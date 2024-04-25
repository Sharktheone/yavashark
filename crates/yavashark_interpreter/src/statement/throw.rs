use swc_ecma_ast::ThrowStmt;

use yavashark_value::error::Error;
use yavashark_value::Value;

use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_throw(&mut self, stmt: &ThrowStmt, scope: &mut Scope) -> Result<Value, Error> {
        todo!()
    }
}
