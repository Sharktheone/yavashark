use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::SeqExpr;
use yavashark_value::error::Error;

impl Context {
    pub fn run_seq(&mut self, stmt: &SeqExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
