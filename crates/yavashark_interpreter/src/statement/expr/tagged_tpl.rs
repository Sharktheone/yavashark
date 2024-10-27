use crate::Interpreter;
use swc_ecma_ast::TaggedTpl;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult};

impl Interpreter {
    pub fn run_tagged_tpl(realm: &mut Realm, stmt: &TaggedTpl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }
}
