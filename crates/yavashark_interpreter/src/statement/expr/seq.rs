use swc_ecma_ast::SeqExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;

impl Context {
    pub fn run_seq(&mut self, stmt: &SeqExpr, scope: &mut crate::scope::Scope) -> Result<Value, Error> {
        todo!()
    }
}