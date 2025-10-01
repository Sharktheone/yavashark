use crate::function::{JSFunction, RawJSFunction};
use std::cell::RefCell;
use swc_common::Span;
use swc_ecma_ast::{
    BlockStmt, Class, ClassMember, Function, MethodKind, Param, ParamOrTsParamProp, PropName,
};
use yavashark_env::value::Obj;
use yavashark_env::{
    scope::Scope, Class as JSClass, ClassInstance, Error, Object, Realm, Res, Value, ValueResult,
    Variable,
};
use yavashark_string::YSString;

use crate::Interpreter;

pub fn create_class(
    realm: &mut Realm,
    stmt: &Class,
    scope: &mut Scope,
    name: String,
) -> Res<(JSClass, Vec<BlockStmt>)> {
    let (mut class, mut proto) = if let Some(class) = &stmt.super_class {
        let super_class = Interpreter::run_expr(realm, class, stmt.span, scope)?;
        let p = super_class
            .get_property(&"prototype".into(), realm)?
            .to_object()?;

        (
            JSClass::with_super(super_class.to_object()?, name.clone())?,
            ClassInstance::new_with_proto(p, name),
        )
    } else {
        (
            JSClass::new(realm, name.clone())?,
            ClassInstance::new(realm, name),
        )
    };

    let mut statics = Vec::new();

    for item in &stmt.body {
        match item {
            ClassMember::Method(method) => {
                let (name, func) =
                    create_method(&method.key, &method.function, scope, realm, stmt.span)?;

                define_method_on_class(
                    name,
                    func,
                    &mut class,
                    &mut proto,
                    method.is_static,
                    false,
                    method.kind,
                );
            }
            ClassMember::Constructor(constructor) => {
                let mut params = Vec::new();

                for param in &constructor.params {
                    let ParamOrTsParamProp::Param(param) = param else {
                        return Err(Error::syn("typescript not supported"));
                    };

                    params.push(param.clone());
                }

                let raw_fn = RawJSFunction {
                    name: RefCell::new("constructor".into()),
                    params,
                    block: constructor.body.clone(),
                    scope: scope.clone(),
                };

                class.set_constructor(raw_fn);
            }
            ClassMember::PrivateMethod(method) => {
                let (name, func) = create_method(
                    &PropName::Ident(method.key.name.as_str().into()),
                    &method.function,
                    scope,
                    realm,
                    stmt.span,
                )?;

                define_method_on_class(
                    name,
                    func,
                    &mut class,
                    &mut proto,
                    method.is_static,
                    true,
                    method.kind,
                )?;
            }

            ClassMember::StaticBlock(s) => {
                statics.push(s.body.clone());
            }

            ClassMember::PrivateProp(o) => {
                let key = o.key.name.as_str();
                let value = if let Some(value) = &o.value {
                    Interpreter::run_expr(realm, value, stmt.span, scope)?
                } else {
                    Value::Undefined
                };

                define_on_class(
                    YSString::from_ref(key).into(),
                    value,
                    &mut class,
                    &mut proto,
                    o.is_static,
                    true,
                );
            }
            ClassMember::Empty(_) => {}
            ClassMember::TsIndexSignature(_) => {
                return Err(Error::syn("TsIndexSignature is not supported"))
            }
            ClassMember::ClassProp(p) => {
                let name = prop_name_to_value(&p.key, realm, p.span, scope)?;

                let value = p.value.as_ref().map_or(
                    Ok(Value::Undefined),
                    (|val| Interpreter::run_expr(realm, val, p.span, scope)),
                )?;

                define_on_class(name, value, &mut class, &mut proto, p.is_static, false)?;
            }
            ClassMember::AutoAccessor(_) => todo!("AutoAccessor"),
        }
    }

    class.set_proto(proto.into_value().into());

    // scope.declare_var(name, this.clone());

    Ok((class, statics))
}

pub fn decl_class_ret(
    realm: &mut Realm,
    stmt: &Class,
    scope: &mut Scope,
    name: String,
) -> ValueResult {
    let (class, statics) = create_class(realm, stmt, scope, name)?;

    let this = class.into_object();

    let mut static_scope = scope.child_object(this.clone())?;

    let this: Value = this.into();

    for static_block in statics {
        Interpreter::run_block_this(realm, &static_block, &mut static_scope, this.copy())?;
    }

    let proto = this.get_property(&"prototype".into(), realm)?;

    proto.define_property("constructor".into(), this.copy());

    Ok(this)
}
pub fn decl_class(realm: &mut Realm, stmt: &Class, scope: &mut Scope, name: String) -> Res {
    let class = decl_class_ret(realm, stmt, scope, name.clone())?;

    scope.declare_var(name, class);
    Ok(())
}

fn create_method(
    name: &PropName,
    func: &Function,
    scope: &mut Scope,
    realm: &mut Realm,
    span: Span,
) -> Res<(Value, Value), Error> {
    let name = prop_name_to_value(name, realm, span, scope)?;

    #[cfg(feature = "vm")]
    {
        if func.is_async || func.is_generator {
            let name_str = name.to_string(realm)?;
            return Ok((
                name,
                (yavashark_bytecode_interpreter::ByteCodeInterpreter::compile_fn(
                    func,
                    name_str.to_string(),
                    scope.clone(),
                    realm,
                )?
                .into()),
            ));
        }
    }

    let func = JSFunction::new(
        name.to_string(realm)?.to_string(),
        func.params.clone(),
        func.body.clone(),
        scope.clone(),
        realm,
    )?;
    Ok((name, func.into()))
}

fn prop_name_to_value(
    name: &PropName,
    realm: &mut Realm,
    span: Span,
    scope: &mut Scope,
) -> ValueResult {
    Ok(match name {
        PropName::Ident(ident) => ident.sym.to_string().into(),
        PropName::Computed(computed) => {
            let expr = &computed.expr;
            Interpreter::run_expr(realm, expr, span, scope)?
        }
        PropName::Num(num) => num.value.to_string().into(),
        PropName::Str(str) => str.value.to_string().into(),
        PropName::BigInt(_) => unimplemented!(),
    })
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
        let Value::String(name) = name else {
            return Err(Error::new(
                "Private method name must be a string (how tf did you get here?)",
            ));
        };

        if is_static {
            class.define_private_field(name.to_string(), value);
        } else {
            proto.define_private_field(name.to_string(), value);
        }

        return Ok(());
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

fn define_method_on_class(
    name: Value,
    value: Value,
    class: &mut JSClass,
    proto: &mut ClassInstance,
    is_static: bool,
    is_private: bool,
    kind: MethodKind,
) -> Res {
    if is_private {
        let Value::String(name) = name else {
            return Err(Error::new(
                "Private method name must be a string (how tf did you get here?)",
            ));
        };

        let key = name.to_string();

        match kind {
            MethodKind::Getter => {
                if is_static {
                    class.define_private_getter(key, value)?;
                } else {
                    proto.define_private_getter(key, value)?;
                }
            }
            MethodKind::Setter => {
                if is_static {
                    class.define_private_setter(key, value)?;
                } else {
                    proto.define_private_setter(key, value)?;
                }
            }
            MethodKind::Method => {
                if is_static {
                    class.define_private_method(key, value);
                } else {
                    proto.define_private_method(key, value);
                }
            }
        }

        return Ok(());
    } else if is_static {
        if name == Value::String("prototype".into()) {
            return Err(Error::new(
                "Classes may not have a static property named 'prototype'",
            ));
        }

        match kind {
            MethodKind::Getter => class.define_getter(name, value),
            MethodKind::Setter => class.define_setter(name, value),
            MethodKind::Method => class.define_variable(name, Variable::write_config(value)),
        };
    } else {
        match kind {
            MethodKind::Getter => proto.define_getter(name, value),
            MethodKind::Setter => proto.define_setter(name, value),
            MethodKind::Method => proto.define_variable(name, Variable::write_config(value)),
        };
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
            r#"
            class A {
                constructor(a){
                    console.log("A constructor called") 
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
                
                ee = 55;
                
                static na = "GE";   
            }

            class B extends A {
                constructor(a, b){
                    super(a);
                    console.log("Weee"); 
                    this.b = b;
                    console.log("wooooo"); 
                }
                
                e = 99;  
                
                static n = "HE";  
            }
            
            
            
            console.log(A)
            console.log(B)
            console.log(A.prototype)
            console.log(B.prototype) 
            console.log(A.__proto__)
            console.log(B.__proto__)
            
            new B(1, 2);
            "#,
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
