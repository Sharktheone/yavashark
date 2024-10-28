use crate::class::decl_class;
use swc_ecma_ast::ClassDecl;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res};

use crate::Interpreter;

impl Interpreter {
    pub fn decl_class(realm: &mut Realm, stmt: &ClassDecl, scope: &mut Scope) -> Res {
        let name = stmt.ident.sym.to_string();

        decl_class(realm, &stmt.class, scope, name)
    }
}
