use crate::Interpreter;
use swc_ecma_ast::Decl;
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, Value};
use crate::statement::decl::var::Variable;

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

    pub fn run_decl_ret(realm: &mut Realm, stmt: &Decl, scope: &mut Scope) -> Res<DeclRet> {
        let (name, value) = match stmt {
            Decl::Class(c) => Self::decl_class_ret(realm, c, scope)?,
            Decl::Fn(f) => Self::decl_fn_ret(realm, f, scope)?,
            Decl::Var(v) => {
                let values = Self::decl_var_ret(realm, v, scope)?;
                
                return Ok(DeclRet::Var(values));
                
            },
            Decl::Using(u) => Self::decl_using_ret(realm, u, scope)?,
            _ => return Err(Error::new("Unsupported declaration")),
        };
        
        Ok(DeclRet::Single(name, value))
    }
}
