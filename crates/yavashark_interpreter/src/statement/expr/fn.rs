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
            .map_or(String::new(), |i| i.sym.to_string());

        if stmt.function.is_async || stmt.function.is_generator {
            #[cfg(feature = "vm")]
            let function = yavashark_bytecode_interpreter::ByteCodeInterpreter::compile_fn(
                &stmt.function,
                name,
                fn_scope.clone(),
                realm,
            )?;

            #[cfg(feature = "vm")]
            if let Some(ident) = &stmt.ident {
                fn_scope.declare_var(ident.sym.to_string(), function.clone().into(), realm);
            }

            #[cfg(feature = "vm")]
            return Ok(function.into());
        }

        let function = JSFunction::new(
            name,
            stmt.function.params.clone(),
            stmt.function.body.clone(),
            fn_scope.clone(),
            realm,
        )?;

        if let Some(ident) = &stmt.ident {
            fn_scope.declare_var(ident.sym.to_string(), function.clone().into(), realm);
        }

        Ok(function.into())
    }
}
