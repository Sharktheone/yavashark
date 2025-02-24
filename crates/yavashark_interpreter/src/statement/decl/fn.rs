use std::any::{type_name_of_val, Any, TypeId};
use std::cell::RefCell;
use std::mem;
use crate::function::{JSFunction, OptimizedJSFunction};
use crate::Interpreter;
use swc_ecma_ast::FnDecl;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, Res, optimizer::OptimFunction};
use yavashark_env::optimizer::FunctionCode;
use yavashark_value::AsAny;

impl Interpreter {
    pub fn decl_fn(realm: &mut Realm, stmt: &FnDecl, scope: &mut Scope) -> Res {
        let mut fn_scope = Scope::with_parent(scope)?;

        fn_scope.state_set_function()?;
        
        let block = stmt.function.body.as_ref().map(|block| {
            let boxed: Box<dyn FunctionCode> = Box::new(OptimizedJSFunction {
                block: block.clone()
            });
            
            RefCell::new(boxed)
        });

        let name = stmt.ident.sym.to_string();
        let function = OptimFunction::new(
            name.clone(),
            stmt.function.params.clone(),
            block,
            fn_scope,
            realm,
        );
        scope.declare_var(name, function.into());

        Ok(())
    }
}
