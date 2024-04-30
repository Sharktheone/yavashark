use swc_ecma_ast::FnDecl;

use crate::context::Context;
use crate::scope::Scope;
use crate::{Function, Res};

impl Context {
    pub fn decl_fn(&mut self, stmt: &FnDecl, scope: &mut Scope) -> Res {
        let name = stmt.ident.sym.to_string();
        let function = Function::JS(stmt.function.params.clone(), stmt.function.body.clone());
        let function_obj = function.into();
        scope.declare_var(name, function_obj);
        
        Ok(())
    }
}
