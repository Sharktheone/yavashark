use crate::Interpreter;
use crate::function::JSFunction;
use std::rc::Rc;
use swc_common::Spanned;
use swc_ecma_ast::{ObjectLit, Param, Prop, PropName, PropOrSpread};
use yavashark_env::scope::Scope;
use yavashark_env::value::property_key::IntoPropertyKey;
use yavashark_env::{ControlFlow, Error, InternalPropertyKey, Object, Realm, RuntimeResult, Value};
use yavashark_string::YSString;

impl Interpreter {
    pub fn run_object(realm: &mut Realm, stmt: &ObjectLit, scope: &mut Scope) -> RuntimeResult {
        let mut obj = Object::new(realm);

        for prop in &stmt.props {
            match prop {
                PropOrSpread::Spread(spread) => {
                    let expr = Self::run_expr(realm, &spread.expr, spread.dot3_token, scope)?;

                    if let Ok(props) = expr.properties(realm) {
                        for (name, value) in props {
                            obj.define_property(name.into(), value, realm);
                        }
                    }
                }

                PropOrSpread::Prop(prop) => {
                    match &**prop {
                        Prop::Shorthand(ident) => {
                            let name = ident.sym.as_str();
                            let value = scope.resolve(name, realm)?.ok_or_else(|| {
                                ControlFlow::error_reference(format!("{name} is not defined"))
                            })?;

                            obj.define_property(
                                InternalPropertyKey::String(YSString::from_ref(name)),
                                value,
                                realm,
                            );
                        }
                        Prop::KeyValue(kv) => {
                            let key = Self::run_prop_key(realm, &kv.key, scope)?;

                            let value = Self::run_expr(realm, &kv.value, prop.span(), scope)?;

                            obj.define_property(key, value, realm);
                        }
                        Prop::Assign(assign) => {
                            let key = InternalPropertyKey::String(YSString::from_ref(
                                assign.key.sym.as_str(),
                            ));

                            let value = Self::run_expr(realm, &assign.value, prop.span(), scope)?;

                            obj.define_property(key, value, realm);
                        }

                        Prop::Method(method) => {
                            let key = Self::run_prop_key(realm, &method.key, scope)?;
                            let mut fn_scope = Scope::with_parent(scope)?;

                            fn_scope.state_set_function();

                            let name = key.to_string(); // TODO, what should the name be here? (and wrong to_string function)
                            let function = if method.function.is_async
                                || method.function.is_generator
                            {
                                #[cfg(feature = "vm")]
                                    let f = yavashark_bytecode_interpreter::ByteCodeInterpreter::compile_fn(
                                        &method.function,
                                        name.clone().clone(),
                                        fn_scope,
                                        realm,
                                    )?;

                                #[cfg(not(feature = "vm"))]
                                let f = JSFunction::new(
                                    name.clone().to_string(),
                                    method.function.params.clone(),
                                    method.function.body.clone(),
                                    fn_scope,
                                    realm,
                                )?;

                                f
                            } else {
                                JSFunction::new(
                                    name.clone().clone(),
                                    method.function.params.clone(),
                                    method.function.body.clone(),
                                    fn_scope,
                                    realm,
                                )?
                            };

                            let value = function.into();

                            obj.define_property(key, value, realm);
                        }
                        Prop::Setter(set) => {
                            let key = Self::run_prop_key(realm, &set.key, scope)?;

                            let param = Param::from((*set.param).clone());
                            let params = vec![param];

                            let mut fn_scope = Scope::with_parent(scope)?;

                            fn_scope.state_set_function()?;

                            let func = JSFunction::new(
                                key.to_string(),
                                params,
                                set.body.clone(),
                                fn_scope,
                                realm,
                            )?;

                            obj.define_setter(key, func, realm)?;
                        }
                        Prop::Getter(get) => {
                            let key = Self::run_prop_key(realm, &get.key, scope)?;

                            let mut fn_scope = Scope::with_parent(scope)?;

                            fn_scope.state_set_function()?;

                            let func = JSFunction::new(
                                key.to_string(),
                                vec![],
                                get.body.clone(),
                                fn_scope,
                                realm,
                            )?;

                            obj.define_getter(key, func, realm)?;
                        }
                    }
                }
            }
        }

        Ok(Value::Object(obj))
    }

    pub fn run_prop_name(realm: &mut Realm, prop: &PropName, scope: &mut Scope) -> RuntimeResult {
        Ok(match prop {
            PropName::Ident(ident) => Value::String(YSString::from_ref(&ident.sym)),
            PropName::Str(str_) => {
                str_.value.as_str().map_or_else(|| {
                    let utf16_units = str_.value.to_ill_formed_utf16();
                    Value::String(YSString::from_utf16_iter(utf16_units))
                },
                |s|
                    Value::String(YSString::from_ref(s))
                )

            }
            PropName::Num(num) => Value::Number(num.value),
            PropName::Computed(expr) => Self::run_expr(realm, &expr.expr, expr.span, scope)?,
            PropName::BigInt(b) => Value::BigInt(Rc::new((*b.value).clone())),
        })
    }

    fn run_prop_key(
        realm: &mut Realm,
        prop: &PropName,
        scope: &mut Scope,
    ) -> Result<InternalPropertyKey, ControlFlow> {
        match prop {
            PropName::Ident(ident) => Ok(InternalPropertyKey::String(YSString::from_ref(
                ident.sym.as_str(),
            ))),
            PropName::Str(str_) => {
                str_.value.as_str().map_or_else(|| {
                    let utf16_units = str_.value.to_ill_formed_utf16();
                    Ok(InternalPropertyKey::String(YSString::from_utf16_iter(utf16_units)))
                },
                |s|
                    Ok(InternalPropertyKey::String(YSString::from_ref(s)))
                )
            }
            PropName::Num(num) => Ok(InternalPropertyKey::from_float(num.value)),
            PropName::Computed(expr) => {
                let value = Self::run_expr(realm, &expr.expr, expr.span, scope)?;
                Ok(value.into_internal_property_key(realm)?)
            }
            PropName::BigInt(b) => Ok(InternalPropertyKey::String(YSString::from_ref(&b.value.to_string()))),
        }
    }
}
