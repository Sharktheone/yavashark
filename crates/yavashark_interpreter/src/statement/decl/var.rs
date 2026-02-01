use std::iter;
use swc_common::DUMMY_SP;
use swc_ecma_ast::{VarDecl, VarDeclKind};
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, Value};

use crate::Interpreter;

#[derive(Debug)]
pub enum Variable {
    Var(String, Value),
    Let(String, Value),
    Const(String, Value),
}

impl Interpreter {
    pub fn decl_var(realm: &mut Realm, stmt: &VarDecl, scope: &mut Scope) -> Res {
        let cb = |scope: &mut Scope, var, realm: &mut Realm| {
            match var {
                Variable::Var(name, value) => {
                    scope.declare_global_var(name, value, realm);
                }
                Variable::Let(name, value) => {
                    scope.declare_var(name, value, realm);
                }
                Variable::Const(name, value) => {
                    scope.declare_read_only_var(name, value, realm);
                }
            }

            Ok(())
        };

        Self::decl_var_cb(realm, stmt, scope, cb)
    }

    pub fn decl_var_ret(
        realm: &mut Realm,
        stmt: &VarDecl,
        scope: &mut Scope,
    ) -> Res<Vec<Variable>> {
        let mut vars = Vec::with_capacity(stmt.decls.len());

        let cb = |scope: &mut Scope, var, realm: &mut Realm| {
            vars.push(var);
            Ok(())
        };

        Self::decl_var_cb(realm, stmt, scope, cb)?;

        Ok(vars)
    }

    pub fn decl_var_cb(
        realm: &mut Realm,
        stmt: &VarDecl,
        scope: &mut Scope,
        mut cb: impl FnMut(&mut Scope, Variable, &mut Realm) -> Res,
    ) -> Res {
        match stmt.kind {
            VarDeclKind::Var => {
                for decl in &stmt.decls {
                    let init = &decl.init;
                    if let Some(init) = init {
                        let value = Self::run_expr(realm, init, stmt.span, scope)?;

                        let named = Self::expr_should_be_named(init);

                        Self::run_pat_internal(
                            realm,
                            &decl.name,
                            scope,
                            &mut iter::once(value),
                            DUMMY_SP,
                            named,
                            &mut |scope, name, value, realm| {
                                let var = Variable::Var(name, value);

                                cb(scope, var, realm)
                            },
                        )?;
                    } else {
                        Self::run_pat_internal(
                            realm,
                            &decl.name,
                            scope,
                            &mut iter::once(Value::Undefined),
                            DUMMY_SP,
                            false,
                            &mut |scope, name, value, realm| {
                                let var = Variable::Var(name, value);

                                cb(scope, var, realm)
                            },
                        )?;
                    }
                }

                Ok(())
            }
            VarDeclKind::Let => {
                for decl in &stmt.decls {
                    let init = &decl.init;
                    if let Some(init) = init {
                        let value = Self::run_expr(realm, init, stmt.span, scope)?;

                        let named = Self::expr_should_be_named(init);

                        Self::run_pat_internal(
                            realm,
                            &decl.name,
                            scope,
                            &mut iter::once(value),
                            DUMMY_SP,
                            named,
                            &mut |scope, name, value, realm| {
                                let var = Variable::Let(name, value);

                                cb(scope, var, realm)
                            },
                        )?;
                    } else {
                        Self::run_pat_internal(
                            realm,
                            &decl.name,
                            scope,
                            &mut iter::once(Value::Undefined),
                            DUMMY_SP,
                            false,
                            &mut |scope, name, value, realm| {
                                let var = Variable::Let(name, value);

                                cb(scope, var, realm)
                            },
                        )?;
                    }
                }
                Ok(())
            }
            VarDeclKind::Const => {
                for decl in &stmt.decls {
                    let init = &decl.init;
                    if let Some(init) = init {
                        let value = Self::run_expr(realm, init, stmt.span, scope)?;

                        let named = Self::expr_should_be_named(init);

                        Self::run_pat_internal(
                            realm,
                            &decl.name,
                            scope,
                            &mut iter::once(value),
                            DUMMY_SP,
                            named,
                            &mut |scope, name, value, realm| {
                                let var = Variable::Const(name, value);

                                cb(scope, var, realm)
                            },
                        )?;
                    } else {
                        return Err(Error::new("Const declaration must have an initializer"));
                    }
                }
                Ok(())
            }
        }
    }
}
