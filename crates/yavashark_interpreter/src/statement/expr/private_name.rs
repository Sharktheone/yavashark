use crate::Interpreter;
use swc_ecma_ast::PrivateName;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_private_name(
        realm: &mut Realm,
        stmt: &PrivateName,
        scope: &mut Scope,
    ) -> RuntimeResult {
        todo!()
    }
}
