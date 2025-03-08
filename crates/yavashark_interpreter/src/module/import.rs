use swc_ecma_ast::ImportDecl;
use yavashark_env::{Realm, RuntimeResult};
use yavashark_env::scope::Scope;
use crate::Interpreter;

impl Interpreter {

    pub fn run_import(realm: &mut Realm, stmt: &ImportDecl, scope: &mut Scope) -> RuntimeResult {
        todo!()
    }

}