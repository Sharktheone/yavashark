mod class;
mod r#fn;
mod using;
mod var;

use crate::{ByteCodegen, Res};
use anyhow::anyhow;
use swc_ecma_ast::{Decl, ExprStmt};

impl ByteCodegen {
    pub fn compile_decl(&mut self, stmt: &Decl) -> Res {
        match stmt {
            Decl::Class(class) => self.compile_class_decl(class),
            Decl::Fn(func) => self.compile_fn_decl(func),
            Decl::Var(var) => self.compile_var_decl(var),
            Decl::Using(using) => self.compile_using_decl(using),

            _ => Err(anyhow!("Typescript not supported")),
        }
    }
}
