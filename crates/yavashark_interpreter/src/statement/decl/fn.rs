use crate::function::{JSFunction, OptimizedJSFunction};
use crate::Interpreter;
use std::any::{type_name_of_val, Any, TypeId};
use std::cell::RefCell;
use std::mem;
use swc_ecma_ast::FnDecl;
use yavashark_bytecode_interpreter::ByteCodeInterpreter;
use yavashark_env::optimizer::FunctionCode;
use yavashark_env::scope::Scope;
use yavashark_env::{optimizer::OptimFunction, Realm, Res, Value};
use yavashark_value::AsAny;

impl Interpreter {
    pub fn decl_fn_ret(
        realm: &mut Realm,
        stmt: &FnDecl,
        scope: &mut Scope,
    ) -> Res<(String, Value)> {
        let mut fn_scope = Scope::with_parent(scope)?;

        fn_scope.state_set_function()?;

        // let block = stmt.function.body.as_ref().map(|block| {
        //     let boxed: Box<dyn FunctionCode> = Box::new(OptimizedJSFunction {
        //         block: block.clone(),
        //     });
        //
        //     RefCell::new(boxed)
        // });

        let name = stmt.ident.sym.to_string();
        let function = if stmt.function.is_async || stmt.function.is_generator {
            ByteCodeInterpreter::compile_fn(&stmt.function, name.clone(), fn_scope, realm)?
        } else {
            JSFunction::new(
                name.clone(),
                stmt.function.params.clone(),
                stmt.function.body.clone(),
                fn_scope,
                realm,
            )
        };

        Ok((name, function.into()))
    }

    pub fn decl_fn(realm: &mut Realm, stmt: &FnDecl, scope: &mut Scope) -> Res {
        let (name, function) = Self::decl_fn_ret(realm, stmt, scope)?;

        scope.declare_var(name, function);

        Ok(())
    }
}
