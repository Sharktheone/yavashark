use swc_ecma_ast::{Pat, VarDecl, VarDeclKind};
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, Value, Result};

use crate::Interpreter;



pub enum Variable {
    Var(String, Value),
    Let(String, Value),
    Const(String, Value),
}

impl Interpreter {
    pub fn decl_var(realm: &mut Realm, stmt: &VarDecl, scope: &mut Scope) -> Res {
        let cb = |scope: &mut Scope, var| {
            match var {
                Variable::Var(name, value) => {
                    scope.declare_global_var(name, value);
                }
                Variable::Let(name, value) => {
                    scope.declare_var(name, value);
                }
                Variable::Const(name, value) => {
                    scope.declare_read_only_var(name, value);
                }
            }

            Ok(())
        };

        Self::decl_var_cb(realm, stmt, scope, cb)
    }

    pub fn decl_var_ret(realm: &mut Realm, stmt: &VarDecl, scope: &mut Scope) -> Result<Vec<Variable>> {
        let mut vars = Vec::with_capacity(stmt.decls.len());

        let cb = |scope: &mut Scope, var| {
            vars.push(var);
            Ok(())
        };

        Self::decl_var_cb(realm, stmt, scope, cb)?;

        Ok(vars)

    }

    pub fn decl_var_cb(realm: &mut Realm, stmt: &VarDecl, scope: &mut Scope, mut cb: impl FnMut(&mut Scope, Variable) -> Res) -> Res {
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

                        let var = Variable::Var(id.sym.to_string(), value);

                        cb(scope, var)?;
                    } else {
                        let var = Variable::Var(id.sym.to_string(), Value::Undefined);

                        cb(scope, var)?;
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
                        let var = Variable::Let(id.sym.to_string(), value);

                        cb(scope, var)?;
                    } else {
                        let var = Variable::Let(id.sym.to_string(), Value::Undefined);

                        cb(scope, var)?;
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

                        let var = Variable::Const(id.sym.to_string(), value);

                        cb(scope, var)?;
                    } else {
                        return Err(Error::new("Const declaration must have an initializer"));
                    }
                }
                Ok(())
            }
        }
    }
}
