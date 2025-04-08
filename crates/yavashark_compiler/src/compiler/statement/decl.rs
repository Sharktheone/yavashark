mod class;
mod fn_;
mod using;
mod var;

use crate::{Compiler, Res};
use anyhow::anyhow;
use swc_ecma_ast::Decl;

impl Compiler {
    pub fn compile_decl(&mut self, d: &Decl) -> Res {
        match d {
            Decl::Class(class) => self.decl_class(class),
            Decl::Fn(fn_decl) => self.decl_fn(fn_decl),
            Decl::Var(var) => self.decl_var(var),
            Decl::Using(using) => self.decl_using(using),

            _ => Err(anyhow!("Typescript not supported")),
        }
    }
}
