use swc_ecma_ast::tplExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_tpl(&mut self, stmt: &tplExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}