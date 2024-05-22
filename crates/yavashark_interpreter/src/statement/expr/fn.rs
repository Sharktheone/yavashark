use crate::context::Context;
use crate::scope::Scope;
use crate::{Error, JSFunction};
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::FnExpr;

impl Context {
    pub fn run_fn(&mut self, stmt: &FnExpr, scope: &mut Scope) -> RuntimeResult {
        let mut fn_scope = Scope::with_parent(scope);

        fn_scope.state_set_function();

        let name = stmt.ident.as_ref().map_or("anonymous".to_string(), |i| i.sym.to_string());
        
        let function = JSFunction::new(
            name,
            stmt.function.params.clone(),
            stmt.function.body.clone(),
            fn_scope,
            self,
        );

        Ok(Value::Function(function))
    }
}
