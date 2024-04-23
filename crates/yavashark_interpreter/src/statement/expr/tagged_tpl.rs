use swc_ecma_ast::tagged_tplExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_tagged_tpl(&mut self, stmt: &tagged_tplExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}