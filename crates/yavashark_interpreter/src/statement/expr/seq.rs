use swc_ecma_ast::SeqExpr;

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;

impl Context {
    pub fn run_seq(&mut self, stmt: &SeqExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
