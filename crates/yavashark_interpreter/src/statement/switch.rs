use swc_ecma_ast::SwitchStmt;

use yavashark_value::error::Error;
use yavashark_value::Value;

use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_switch(&mut self, stmt: &SwitchStmt, scope: &mut Scope) -> Result<Value, Error> {
        todo!()
    }
}
