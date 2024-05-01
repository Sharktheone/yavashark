mod class;
mod r#fn;
mod using;
mod var;

use crate::context::Context;
use crate::scope::Scope;
use crate::Value;
use crate::{ControlFlow, Res, RuntimeResult};
use swc_ecma_ast::Decl;
use crate::Error;

impl Context {
    pub fn run_decl(&mut self, stmt: &Decl, scope: &mut Scope) -> Res {
        match stmt {
            Decl::Class(c) => self.decl_class(c, scope),
            Decl::Fn(f) => self.decl_fn(f, scope),
            Decl::Var(v) => self.decl_var(v, scope),
            Decl::Using(u) => self.decl_using(u, scope),
            _ => Err(Error::new("Unsupported declaration".to_string())),
        }
    }
}
