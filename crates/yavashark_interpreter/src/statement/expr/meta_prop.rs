use crate::Interpreter;
use swc_ecma_ast::MetaPropExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_meta_prop(
        realm: &mut Realm,
        stmt: &MetaPropExpr,
        scope: &mut Scope,
    ) -> RuntimeResult {
        todo!()
    }
}
