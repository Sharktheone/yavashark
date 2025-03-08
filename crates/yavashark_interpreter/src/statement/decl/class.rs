use crate::class::{decl_class, decl_class_ret};
use swc_ecma_ast::ClassDecl;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res, Result, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn decl_class(realm: &mut Realm, stmt: &ClassDecl, scope: &mut Scope) -> Res {
        let name = stmt.ident.sym.to_string();

        decl_class(realm, &stmt.class, scope, name)
    }
    
    pub fn decl_class_ret(realm: &mut Realm, stmt: &ClassDecl, scope: &mut Scope) -> Result<(String, Value)> {
        let name = stmt.ident.sym.to_string();

        let class = decl_class_ret(realm, &stmt.class, scope, name.clone())?;
        
        
        Ok((name, class))
    }
    
    
}
