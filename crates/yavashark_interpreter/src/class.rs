use swc_ecma_ast::{Class, ClassMember, PropName};
use yavashark_env::{scope::Scope, Class as JSClass, Context, Res};
use yavashark_value::Obj;

use crate::{function::JSFunction, Interpreter};





pub fn decl_class(ctx: &mut Context, stmt: &Class, scope: &mut Scope) -> Res {
    let mut class = if let Some(class) = &stmt.super_class {
        let super_class = Interpreter::run_expr(ctx, class, stmt.span, scope)?;
        JSClass::new_with_proto(super_class)
    } else {
        JSClass::new(ctx)
    };

    for item in &stmt.body {

        match item {
            ClassMember::Method(method) => {
                
                let name = match &method.key {
                    PropName::Ident(ident) => ident.sym.to_string().into(),
                    PropName::Computed(computed) => {
                        let expr = &computed.expr;
                        Interpreter::run_expr(ctx, expr, stmt.span, scope)?
                    }
                    PropName::Num(num) => num.value.to_string().into(),
                    PropName::Str(str) => str.value.to_string().into(),
                    PropName::BigInt(_) => unimplemented!(),
                };
                
                
                
                let func = &method.function;
                
                let func = JSFunction::new(name.to_string(), func.params.clone(), func.body.clone(), scope.clone(), ctx);
                
                class.define_property(name, func.into());
            },

            ClassMember::Constructor(constructor) => {
                todo!()
            },

            _ => {}
        }
    }

        Ok(())
}
