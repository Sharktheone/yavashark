use swc_ecma_ast::UsingDecl;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res, Value, Result};

use crate::Interpreter;

impl Interpreter {
    pub fn decl_using(realm: &mut Realm, stmt: &UsingDecl, scope: &mut Scope) -> Res {
        todo!()
    }
    
    pub fn decl_using_ret(realm: &mut Realm, stmt: &UsingDecl, scope: &mut Scope) -> Result<(String, Value)> {
        todo!()
    }
}
