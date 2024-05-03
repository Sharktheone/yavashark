use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::SeqExpr;

impl Context {
    pub fn run_seq(&mut self, stmt: &SeqExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
