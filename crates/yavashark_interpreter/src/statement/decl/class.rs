use swc_ecma_ast::ClassDecl;
use yavashark_env::{Context, Res};
use yavashark_env::scope::Scope;

use crate::Interpreter;

impl Interpreter {
    pub fn decl_class(ctx: &mut Context, stmt: &ClassDecl, scope: &mut Scope) -> Res {
        todo!()
    }
}
