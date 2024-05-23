use swc_ecma_ast::FnDecl;

use crate::context::Context;
use crate::scope::Scope;
use crate::{JSFunction, Res};

impl Context {
    pub fn decl_fn(&mut self, stmt: &FnDecl, scope: &mut Scope) -> Res {
        let mut fn_scope = Scope::with_parent(scope);

        fn_scope.state_set_function();

        let name = stmt.ident.sym.to_string();
        let function = JSFunction::new(
            name.clone(),
            stmt.function.params.clone(),
            stmt.function.body.clone(),
            fn_scope,
            self,
        );
        scope.declare_var(name, function.into());

        Ok(())
    }
}
