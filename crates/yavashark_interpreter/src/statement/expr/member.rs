use swc_ecma_ast::MemberExpr;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_member(&mut self, stmt: &MemberExpr, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}