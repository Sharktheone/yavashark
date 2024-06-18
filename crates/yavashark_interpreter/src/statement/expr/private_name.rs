use crate::Interpreter;
use swc_ecma_ast::PrivateName;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_private_name(
        ctx: &mut Context,
        stmt: &PrivateName,
        scope: &mut Scope,
    ) -> RuntimeResult {
        todo!()
    }
}
