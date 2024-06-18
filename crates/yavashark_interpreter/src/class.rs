use swc_common::Span;
use swc_ecma_ast::{BlockStmt, Class, ClassMember, Function, Param, PropName, Stmt};
use yavashark_env::{scope::Scope, Class as JSClass, Context, Error, Res, Value, ValueResult};
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
                let (name, func) = create_method(&method.key, method.function.params.clone(), method.function.body.clone(), scope, ctx, stmt.span)?;

                class.define_property(name, func);
            }
            ClassMember::Constructor(constructor) => {
                let params = Vec::new(); //TODO

                let (name, func) = create_method(&PropName::Ident("constructor".into()), params, constructor.body.clone(), scope, ctx, stmt.span)?;

                class.define_property(name, func);
            }
            ClassMember::PrivateMethod(method) => {
                let (name, func) = create_method(&PropName::Ident(format!("#{}", method.key.id).as_str().into()), method.function.params.clone(), method.function.body.clone(), scope, ctx, stmt.span)?;

                class.set_private_prop(method.key.id.to_string(), func);
            }

            _ => {}
        }
    }

        Ok(())
}


fn create_method(name: &PropName, params: Vec<Param>, body: Option<BlockStmt>, scope: &mut Scope, ctx: &mut Context, span: Span) -> Result<(Value, Value), Error> {
    let name = match name {
        PropName::Ident(ident) => ident.sym.to_string().into(),
        PropName::Computed(computed) => {
            let expr = &computed.expr;
            Interpreter::run_expr(ctx, expr, span, scope)?
        }
        PropName::Num(num) => num.value.to_string().into(),
        PropName::Str(str) => str.value.to_string().into(),
        PropName::BigInt(_) => unimplemented!(),
    };



    let func = JSFunction::new(name.to_string(), params, body, scope.clone(), ctx);
    Ok((name, func.into()))
}
