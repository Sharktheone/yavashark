use swc_ecma_ast::SuperPropExpr;
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_super_prop(&mut self, stmt: &SuperPropExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}