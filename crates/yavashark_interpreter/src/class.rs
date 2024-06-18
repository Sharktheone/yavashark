use swc_common::Span;
use swc_ecma_ast::{BlockStmt, Class, ClassMember, Param, PropName};

use yavashark_env::{Class as JSClass, Context, Error, Res, scope::Scope, Value};
use yavashark_value::Obj;

use crate::{function::JSFunction, Interpreter};

pub fn decl_class(ctx: &mut Context, stmt: &Class, scope: &mut Scope) -> Res {
    let mut class = if let Some(class) = &stmt.super_class {
        let super_class = Interpreter::run_expr(ctx, class, stmt.span, scope)?;
        JSClass::new_with_proto(super_class)
    } else {
        JSClass::new(ctx)
    };


    let mut statics = Vec::new();

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

            ClassMember::StaticBlock(s) => {
                statics.push(s.body.clone());
            }

            ClassMember::PrivateProp(o) => {
                let key = format!("#{}", o.key.id);
                let value = if let Some(value) = &o.value {
                    Interpreter::run_expr(ctx, value, stmt.span, scope)?
                } else {
                    Value::Undefined
                };
                
                class.set_private_prop(key, value);
            }
            ClassMember::Empty(_) => {}
            ClassMember::TsIndexSignature(_) => return Err(Error::syn("TsIndexSignature is not supported")),
            ClassMember::ClassProp(_) => return Err(Error::syn("ClassProp is not supported")),
            ClassMember::AutoAccessor(_) => todo!("AutoAccessor"),
        }
    }
    
    //TODO: handle static properties
    
    
    for static_block in statics {
        Interpreter::run_block(ctx, &static_block, scope)?; //TODO: what does `this` refer to, and how does scoping look like?
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
