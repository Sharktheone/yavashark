use crate::Interpreter;
use swc_ecma_ast::Decl;
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res};

mod class;
mod r#fn;
mod using;
mod var;

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
}
