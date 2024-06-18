use crate::Interpreter;
use swc_ecma_ast::Decl;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, Error, Res};

mod class;
mod r#fn;
mod using;
mod var;

impl Interpreter {
    pub fn run_decl(ctx: &mut Context, stmt: &Decl, scope: &mut Scope) -> Res {
        match stmt {
            Decl::Class(c) => Self::decl_class(ctx, c, scope),
            Decl::Fn(f) => Self::decl_fn(ctx, f, scope),
            Decl::Var(v) => Self::decl_var(ctx, v, scope),
            Decl::Using(u) => Self::decl_using(ctx, u, scope),
            _ => Err(Error::new("Unsupported declaration")),
        }
    }
}
