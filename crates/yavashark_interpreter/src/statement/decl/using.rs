use swc_ecma_ast::UsingDecl;
use yavashark_env::{Context, Res};
use yavashark_env::scope::Scope;

use crate::Interpreter;

impl Interpreter {
    pub fn decl_using(ctx: &mut Context, stmt: &UsingDecl, scope: &mut Scope) -> Res {
        todo!()
    }
}
