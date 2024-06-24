use swc_ecma_ast::ClassDecl;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, Res};
use crate::class::decl_class;

use crate::Interpreter;

impl Interpreter {
    pub fn decl_class(ctx: &mut Context, stmt: &ClassDecl, scope: &mut Scope) -> Res {
        let name = stmt.ident.sym.to_string();
        
        decl_class(ctx, &stmt.class, scope, name)
    }
}
