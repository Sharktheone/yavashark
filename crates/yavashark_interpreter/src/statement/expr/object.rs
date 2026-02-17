use crate::function::JSFunction;
use crate::Interpreter;
use std::rc::Rc;
use swc_common::Spanned;
use swc_ecma_ast::{ObjectLit, Param, Prop, PropName, PropOrSpread};
use yavashark_env::scope::Scope;
use yavashark_env::value::property_key::IntoPropertyKey;
use yavashark_env::{ControlFlow, Error, Object, Realm, RuntimeResult, Value};
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
                            let name = ident.sym.to_string();
                            let value = scope.resolve(&name, realm)?.ok_or(
                                ControlFlow::error_reference(format!("{name} is not defined")),
                            )?;

                            obj.define_property(name.into(), value, realm);
                        }
                        Prop::KeyValue(kv) => {
                            let key = Self::run_prop_name(realm, &kv.key, scope)?
                                .into_internal_property_key(realm)?;

                            let value = Self::run_expr(realm, &kv.value, prop.span(), scope)?;

                            obj.define_property(key, value, realm);
                        }
                        Prop::Assign(assign) => {
                            let key = assign.key.sym.to_string();

                            let value = Self::run_expr(realm, &assign.value, prop.span(), scope)?;

                            obj.define_property(key.into(), value, realm);
                        }

                        Prop::Method(method) => {
                            let key = Self::run_prop_name(realm, &method.key, scope)?
                                .into_internal_property_key(realm)?;
                            let mut fn_scope = Scope::with_parent(scope)?;

                            fn_scope.state_set_function();

                            let name = key.to_string(); // TODO, what should the name be here? (and wrong to_string function)
                            let function = if method.function.is_async
                                || method.function.is_generator
                            {
                                #[cfg(feature = "vm")]
                                    let f = yavashark_bytecode_interpreter::ByteCodeInterpreter::compile_fn(
                                        &method.function,
                                        name.clone().to_string(),
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
                                    name.clone().to_string(),
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
                            let key = Self::run_prop_name(realm, &set.key, scope)?
                                .into_internal_property_key(realm)?;

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
                            let key = Self::run_prop_name(realm, &get.key, scope)?
                                .into_internal_property_key(realm)?;

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
                if let Some(s) = str_.value.as_str() {
                    Value::String(YSString::from_ref(s))
                } else {
                    let utf16_units: Vec<u16> = str_.value.to_ill_formed_utf16().collect();
                    Value::String(YSString::from_utf16(&utf16_units))
                }
            }
            PropName::Num(num) => Value::Number(num.value),
            PropName::Computed(expr) => Self::run_expr(realm, &expr.expr, expr.span, scope)?,
            PropName::BigInt(b) => Value::BigInt(Rc::new((*b.value).clone())),
        })
    }
}
