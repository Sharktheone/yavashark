use crate::function::JSFunction;
use crate::Interpreter;
use swc_ecma_ast::FnDecl;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res};

impl Interpreter {
    pub fn decl_fn(realm: &mut Realm, stmt: &FnDecl, scope: &mut Scope) -> Res {
        let mut fn_scope = Scope::with_parent(scope)?;

        fn_scope.state_set_function()?;

        let name = stmt.ident.sym.to_string();
        let function = JSFunction::new(
            name.clone(),
            stmt.function.params.clone(),
            stmt.function.body.clone(),
            fn_scope,
            realm,
        );
        scope.declare_var(name, function.into());

        Ok(())
    }
}
