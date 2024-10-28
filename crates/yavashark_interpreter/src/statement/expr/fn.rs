use crate::function::JSFunction;
use crate::Interpreter;
use swc_ecma_ast::FnExpr;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult};

impl Interpreter {
    pub fn run_fn(realm: &mut Realm, stmt: &FnExpr, scope: &mut Scope) -> RuntimeResult {
        let mut fn_scope = Scope::with_parent(scope)?;

        fn_scope.state_set_function()?;

        let name = stmt
            .ident
            .as_ref()
            .map_or("anonymous".to_string(), |i| i.sym.to_string());

        let function = JSFunction::new(
            name,
            stmt.function.params.clone(),
            stmt.function.body.clone(),
            fn_scope,
            realm,
        );

        Ok(function.into())
    }
}
