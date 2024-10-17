use swc_common::Span;
use swc_ecma_ast::{BlockStmt, Class, ClassMember, Param, PropName};

use crate::function::JSFunction;
use yavashark_env::{
    scope::Scope, Class as JSClass, ClassInstance, Context, Error, Object, Res, Value,
};
use yavashark_value::Obj;

use crate::Interpreter;

pub fn decl_class(ctx: &mut Context, stmt: &Class, scope: &mut Scope, name: String) -> Res {
    let mut class = if let Some(class) = &stmt.super_class {
        let super_class = Interpreter::run_expr(ctx, class, stmt.span, scope)?;
        JSClass::new_with_proto(super_class)
    } else {
        JSClass::new(ctx)
    };

    let mut proto = ClassInstance::new(ctx);

    let mut statics = Vec::new();

    for item in &stmt.body {
        match item {
            ClassMember::Method(method) => {
                let (name, func) = create_method(
                    &method.key,
                    method.function.params.clone(),
                    method.function.body.clone(),
                    scope,
                    ctx,
                    stmt.span,
                )?;

                define_on_class(name, func, &mut class, &mut proto, method.is_static, false);
            }
            ClassMember::Constructor(constructor) => {
                let params = Vec::new(); //TODO

                let (name, func) = create_method(
                    &PropName::Ident("constructor".into()),
                    params,
                    constructor.body.clone(),
                    scope,
                    ctx,
                    stmt.span,
                )?;

                proto.define_property(name, func);
            }
            ClassMember::PrivateMethod(method) => {
                let (name, func) = create_method(
                    &PropName::Ident(format!("#{}", method.key.name).as_str().into()),
                    method.function.params.clone(),
                    method.function.body.clone(),
                    scope,
                    ctx,
                    stmt.span,
                )?;

                define_on_class(name, func, &mut class, &mut proto, method.is_static, true);
            }

            ClassMember::StaticBlock(s) => {
                statics.push(s.body.clone());
            }

            ClassMember::PrivateProp(o) => {
                let key = format!("#{}", o.key.name);
                let value = if let Some(value) = &o.value {
                    Interpreter::run_expr(ctx, value, stmt.span, scope)?
                } else {
                    Value::Undefined
                };

                define_on_class(key.into(), value, &mut class, &mut proto, false, true);
            }
            ClassMember::Empty(_) => {}
            ClassMember::TsIndexSignature(_) => {
                return Err(Error::syn("TsIndexSignature is not supported"))
            }
            ClassMember::ClassProp(_) => return Err(Error::syn("ClassProp is not supported")),
            ClassMember::AutoAccessor(_) => todo!("AutoAccessor"),
        }
    }

    class.prototype = proto.into_value();

    let this = class.into_value();

    scope.declare_var(name, this.clone());

    for static_block in statics {
        Interpreter::run_block_this(ctx, &static_block, scope, this.copy())?;
    }

    Ok(())
}

fn create_method(
    name: &PropName,
    params: Vec<Param>,
    body: Option<BlockStmt>,
    scope: &mut Scope,
    ctx: &mut Context,
    span: Span,
) -> Result<(Value, Value), Error> {
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

    let func = JSFunction::new(name.to_string(ctx)?, params, body, scope.clone(), ctx);
    Ok((name, func.into()))
}

fn define_on_class(
    name: Value,
    value: Value,
    class: &mut JSClass,
    proto: &mut ClassInstance,
    is_static: bool,
    is_private: bool,
) -> Res {
    if is_private {
        if is_static {
            let Value::String(name) = name else {
                return Err(Error::new(
                    "Private static method name must be a string (how tf did you get here?)",
                ));
            };

            class.set_private_prop(name, value);
        } else {
            let Value::String(name) = name else {
                return Err(Error::new(
                    "Private method name must be a string (how tf did you get here?)",
                ));
            };

            proto.set_private_prop(name, value);
        }
    } else if is_static {
        if name == Value::String("prototype".into()) {
            return Err(Error::new(
                "Classes may not have a static property named 'prototype'",
            ));
        }

        class.define_property(name, value);
    } else {
        proto.define_property(name, value);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_decl_class() {
        use yavashark_env::{test_eval, Value};

        test_eval!(
            r"
            class A {
                constructor(a) {
                    this.a = a;
                }

                static b(){
                    return 2;
                }

                c(){
                    return 3;
                }

                #d(){
                    return 4;
                }

                static #e = 5;
            }

            new A(1);
            ",
            0,
            Vec::<Vec<Value>>::new(),
            object
        );
    }


    #[test]
    fn test_decl_class_with_super() {
        use yavashark_env::{test_eval, Value};

        test_eval!(
            r"
            class A {
                constructor(a){
                    this.a = a;
                }

                static b(){
                    return 2;
                }

                c(){
                    return 3;
                }

                #d(){
                    return 4;
                }

                static #e = 5;
            }

            class B extends A {
                constructor(a, b){
                    super(a);
                    this.b = b;
                }
            }

            new B(1, 2);
            ",
            0,
            Vec::<Vec<Value>>::new(),
            object
        );
    }


    #[test]
    fn test_decl_class_with_static_block() {
        use yavashark_env::{test_eval, Value};

        test_eval!(
            r"
            class A {
                static {
                    this.a = 1;
                }
            }

            A.a;
            ",
            0,
            Vec::<Vec<Value>>::new(),
            Value::Number(1.0)
        );
    }
}
