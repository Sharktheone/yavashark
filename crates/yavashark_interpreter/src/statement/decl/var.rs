use swc_ecma_ast::{Pat, VarDecl, VarDeclKind};
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Error, Res, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn decl_var(realm: &mut Realm, stmt: &VarDecl, scope: &mut Scope) -> Res {
        match stmt.kind {
            VarDeclKind::Var => {
                for decl in &stmt.decls {
                    let id = &decl.name;
                    let Pat::Ident(id) = id else {
                        return Err(Error::new("Pattern is not an identifier"));
                    };

                    let init = &decl.init;
                    if let Some(init) = init {
                        let value = Self::run_expr(realm, init, stmt.span, scope)?;
                        scope.declare_global_var(id.sym.to_string(), value);
                    } else {
                        scope.declare_global_var(id.sym.to_string(), Value::Undefined);
                    }
                }

                Ok(())
            }
            VarDeclKind::Let => {
                for decl in &stmt.decls {
                    let id = &decl.name;
                    let Pat::Ident(id) = id else {
                        return Err(Error::new("Pattern is not an identifier"));
                    };

                    let init = &decl.init;
                    if let Some(init) = init {
                        let value = Self::run_expr(realm, init, stmt.span, scope)?;
                        scope.declare_var(id.sym.to_string(), value);
                    } else {
                        scope.declare_var(id.sym.to_string(), Value::Undefined);
                    }
                }
                Ok(())
            }
            VarDeclKind::Const => {
                for decl in &stmt.decls {
                    let id = &decl.name;
                    let Pat::Ident(id) = id else {
                        return Err(Error::new("Pattern is not an identifier"));
                    };

                    let init = &decl.init;
                    if let Some(init) = init {
                        let value = Self::run_expr(realm, init, stmt.span, scope)?;
                        scope.declare_read_only_var(id.sym.to_string(), value);
                    } else {
                        return Err(Error::new("Const declaration must have an initializer"));
                    }
                }
                Ok(())
            }
        }
    }
}
