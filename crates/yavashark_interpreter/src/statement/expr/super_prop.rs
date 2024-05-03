use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::SuperPropExpr;

impl Context {
    pub fn run_super_prop(&mut self, stmt: &SuperPropExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
