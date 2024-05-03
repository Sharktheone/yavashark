use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::UnaryExpr;

impl Context {
    pub fn run_unary(&mut self, stmt: &UnaryExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
