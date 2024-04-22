use swc_ecma_ast::ExprStmt;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::scope::Scope;

impl Context {
    pub fn run_expr(&mut self, stmt: &ExprStmt, scope: &mut Scope) -> Result<Value, Error> {
        todo!()
    }
}
