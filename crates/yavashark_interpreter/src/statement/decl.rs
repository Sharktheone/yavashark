use crate::statement::decl::var::Variable;
use crate::Interpreter;
use swc_ecma_ast::{Decl, ObjectPatProp, Pat, VarDecl, VarDeclKind};
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, Value};

pub mod class;
pub mod r#fn;
pub mod using;
pub mod var;

pub enum DeclRet {
    Single(String, Value),
    Var(Vec<Variable>),
}

impl Interpreter {
    pub fn run_decl(realm: &mut Realm, stmt: &Decl, scope: &mut Scope) -> Res {
        match stmt {
            Decl::Class(c) => Self::decl_class(realm, c, scope),
            Decl::Fn(f) => Self::decl_fn(realm, f, scope),
            Decl::Var(v) => Self::decl_var(realm, v, scope),
            Decl::Using(u) => Self::decl_using(realm, u, scope),
            _ => Err(Error::new("Unsupported declaration")),
        }
    }

    pub fn hoist_decl(realm: &mut Realm, stmt: &Decl, scope: &mut Scope) -> Res {
        match stmt {
            Decl::Class(c) => scope.hoist(c.ident.to_string()),
            Decl::Fn(f) => Self::decl_fn(realm, f, scope),
            Decl::Var(v) => Self::hoist_var(realm, v, scope),
            Decl::Using(u) => Ok(()),
            _ => Err(Error::new("Unsupported declaration")),
        }
    }

    pub fn run_decl_ret(realm: &mut Realm, stmt: &Decl, scope: &mut Scope) -> Res<DeclRet> {
        let (name, value) = match stmt {
            Decl::Class(c) => Self::decl_class_ret(realm, c, scope)?,
            Decl::Fn(f) => Self::decl_fn_ret(realm, f, scope)?,
            Decl::Var(v) => {
                let values = Self::decl_var_ret(realm, v, scope)?;

                return Ok(DeclRet::Var(values));
            }
            Decl::Using(u) => Self::decl_using_ret(realm, u, scope)?,
            _ => return Err(Error::new("Unsupported declaration")),
        };

        Ok(DeclRet::Single(name, value))
    }

    pub fn hoist_var(realm: &mut Realm, var: &VarDecl, scope: &mut Scope) -> Res {
        let decls = var
            .decls
            .iter()
            .flat_map(|v| pat_idents(&v.name))
            .collect::<Vec<_>>();

        if var.kind == VarDeclKind::Var {
            for decl in decls {
                scope.declare_global_var(decl, Value::Undefined, realm)?;
            }
        } else {
            for decl in decls {
                scope.hoist(decl)?;
            }
        }

        Ok(())
    }
}

fn pat_idents(pat: &Pat) -> Vec<String> {
    match pat {
        Pat::Ident(ident) => vec![ident.id.sym.to_string()],
        Pat::Array(array) => array
            .elems
            .iter()
            .filter_map(|v| v.as_ref())
            .flat_map(pat_idents)
            .collect(),
        Pat::Rest(rest) => pat_idents(&rest.arg),
        Pat::Object(obj) => obj
            .props
            .iter()
            .flat_map(|v| match v {
                ObjectPatProp::KeyValue(kv) => pat_idents(&kv.value),
                ObjectPatProp::Assign(assign) => vec![assign.key.sym.to_string()],
                ObjectPatProp::Rest(rest) => pat_idents(&rest.arg),
            })
            .collect(),
        Pat::Assign(assign) => pat_idents(&assign.left),

        _ => Vec::new(),
    }
}
