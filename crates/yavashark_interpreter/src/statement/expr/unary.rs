use swc_ecma_ast::UnaryExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_unary(&mut self, stmt: &UnaryExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}