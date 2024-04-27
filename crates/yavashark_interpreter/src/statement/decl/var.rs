use swc_ecma_ast::{Pat, VarDecl, VarDeclKind};
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::{Res, RuntimeResult};
use crate::scope::Scope;

impl Context {
    pub fn decl_var(&mut self, stmt: &VarDecl, scope: &mut Scope) -> Res {
        match stmt.kind {
            VarDeclKind::Var => {
                for decl in &stmt.decls {
                    let id = &decl.name;
                    let Pat::Ident(id) = id else {
                        return Err(Error::new("Pattern is not an identifier".to_owned()));
                    };

                    let init = &decl.init;
                    if let Some(init) = init {
                        let value = self.run_expr(init, stmt.span, scope)?;
                        scope.declare_global_var(id.sym.to_string(), value);
                    } else {
                        scope.declare_global_var(id.sym.to_string(), Value::Undefined);
                    }
                }

                Ok(())
            },
            VarDeclKind::Let => {
                for decl in &stmt.decls {
                    let id = &decl.name;
                    let Pat::Ident(id) = id else {
                        return Err(Error::new("Pattern is not an identifier".to_owned()));
                    };

                    let init = &decl.init;
                    if let Some(init) = init {
                        let value = self.run_expr(init, stmt.span, scope)?;
                        scope.declare_var(id.sym.to_string(), value);
                    } else {
                        scope.declare_var(id.sym.to_string(), Value::Undefined);
                    }
                }
                Ok(())
            },
            VarDeclKind::Const => {
                for decl in &stmt.decls {
                    let id = &decl.name;
                    let Pat::Ident(id) = id else {
                        return Err(Error::new("Pattern is not an identifier".to_owned()));
                    };

                    let init = &decl.init;
                    if let Some(init) = init {
                        let value = self.run_expr(init, stmt.span, scope)?;
                        scope.declare_read_only_var(id.sym.to_string(), value);
                    } else {
                        return Err(Error::new("Const declaration must have an initializer".to_owned()));
                    }
                }
                Ok(())
            },
        }
    }
}