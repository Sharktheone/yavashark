use std::iter;
use swc_common::{Span, DUMMY_SP};
use swc_ecma_ast::{Expr, ObjectPatProp, Pat, PropName};

use crate::function::JSFunction;
use crate::statement::expr::ArrowFunction;
use crate::Interpreter;
use yavashark_env::array::Array;
use yavashark_env::scope::Scope;
use yavashark_env::value::property_key::IntoPropertyKey;
use yavashark_env::value::IntoValue;
use yavashark_env::value::Obj;
use yavashark_env::{
    Class, ClassInstance, Error, Object, PropertyKey, Realm, Res, Value, ValueResult,
};
use yavashark_string::YSString;
use yavashark_vm::async_generator::AsyncGeneratorFunction;
use yavashark_vm::generator::GeneratorFunction;

impl Interpreter {
    pub fn run_pat(
        realm: &mut Realm,
        stmt: &Pat,
        scope: &mut Scope,
        value: &mut impl Iterator<Item = Value>,
        mut cb: &mut impl FnMut(&mut Scope, String, Value, &mut Realm) -> Res,
    ) -> Res {
        Self::run_pat_internal(realm, stmt, scope, value, DUMMY_SP, true, cb)
    }

    #[allow(clippy::missing_panics_doc)] //Again, cannot panic in the real world
    pub fn run_pat_internal(
        realm: &mut Realm,
        stmt: &Pat,
        scope: &mut Scope,
        value: &mut impl Iterator<Item = Value>,
        span: Span,
        name: bool,
        mut cb: &mut impl FnMut(&mut Scope, String, Value, &mut Realm) -> Res,
    ) -> Res {
        match stmt {
            Pat::Ident(id) => {
                let value = value.next().unwrap_or(Value::Undefined);

                if name {
                    set_value_name(id.id.sym.as_str(), &value, realm)?;
                }

                cb(scope, id.id.sym.to_string(), value, realm)?;
            }
            Pat::Array(arr) => {
                let mut iter = value
                    .next()
                    .unwrap_or(Value::Undefined)
                    .iter_no_realm(realm)?;

                let mut assert_last = false;
                let mut is_finished = false;

                for elem in &arr.elems {
                    if assert_last {
                        return Err(Error::syn("Rest element must be last element"));
                    }

                    if matches!(elem, Some(Pat::Rest(_))) {
                        #[allow(clippy::unwrap_used)]
                        let rest = elem.as_ref().unwrap(); // Safe to unwrap because of the match above

                        let mut elems = Vec::new();

                        while let Some(res) = iter.next(realm)? {
                            elems.push(res);
                        }

                        if elems.is_empty() && scope.is_strict_mode()? {
                            return Err(Error::reference(
                                "Cannot destructure empty array in strict mode",
                            ));
                        }

                        Self::run_pat(realm, rest, scope, &mut elems.into_iter(), cb)?;
                        let assert_last = true;
                    } else {
                        let next = if let Some(value) = iter.next(realm)? {
                            value
                        } else {
                            is_finished = true;
                            if scope.is_strict_mode()? {
                                return Err(Error::reference(
                                    "Cannot destructure empty array in strict mode",
                                ));
                            }

                            Value::Undefined
                        };

                        if let Some(elem) = elem {
                            Self::run_pat(realm, elem, scope, &mut iter::once(next), cb)?;
                        }
                    }
                }

                if !is_finished {
                    iter.close(realm)?;
                }
            }
            Pat::Rest(rest) => {
                let collect = value.collect::<Vec<_>>();

                let array = Array::with_elements(realm, collect)?.into_value();

                Self::run_pat(realm, &rest.arg, scope, &mut iter::once(array), cb)?;
            }
            Pat::Object(obj) => {
                let mut rest_not_props = Vec::with_capacity(obj.props.len());
                let Some(object) = value.next() else {
                    return Err(Error::ty("Cannot destructure undefined"));
                };

                if object.is_nullish() {
                    return Err(Error::ty("Cannot destructure null or undefined"));
                }

                for prop in &obj.props {
                    match prop {
                        ObjectPatProp::KeyValue(kv) => {
                            let key = Self::prop_name_to_value(realm, &kv.key, scope)?;
                            let value = object.get_property_opt(key.clone(), realm)?;

                            let mut value = if let Some(value) = value {
                                value
                            } else {
                                if scope.is_strict_mode()? {
                                    return Err(Error::reference_error(format!(
                                        "Property {key:?} does not exist on object",
                                    )));
                                }

                                Value::Undefined
                            };

                            Self::run_pat(realm, &kv.value, scope, &mut iter::once(value), cb)?;
                            rest_not_props.push(key);
                        }
                        ObjectPatProp::Assign(assign) => {
                            let key = assign.key.sym.to_string();
                            let mut value = object
                                .get_property_opt(key.clone(), realm)?
                                .unwrap_or(Value::Undefined);

                            if let Some(val_expr) = &assign.value {
                                if value.is_undefined() {
                                    value = Self::run_expr(realm, val_expr, assign.span, scope)?;
                                }

                                let should_be_named = Self::expr_should_be_named(val_expr);

                                if should_be_named {
                                    set_value_name(&key, &value, realm)?;
                                }
                            }

                            cb(scope, key.clone(), value, realm)?;
                            rest_not_props.push(key.into());
                        }
                        ObjectPatProp::Rest(rest) => {
                            let mut rest_props = Vec::new();

                            for (name, value) in object.as_object()?.enum_properties(realm)? {
                                if !rest_not_props.contains(&name) {
                                    rest_props.push((name, value));
                                }
                            }

                            let rest_obj = Object::from_values(rest_props, realm)?;

                            Self::run_pat(
                                realm,
                                &rest.arg,
                                scope,
                                &mut iter::once(rest_obj.into_value()),
                                cb,
                            )?;
                        }
                    }
                }
            }
            Pat::Assign(assign) => {
                let val = value.next().unwrap_or(Value::Undefined);

                let val = if val.is_undefined() {
                    Self::run_expr(realm, &assign.right, assign.span, scope)?
                } else {
                    val
                };

                let should_be_named = Self::expr_should_be_named(&assign.right);

                Self::run_pat_internal(
                    realm,
                    &assign.left,
                    scope,
                    &mut iter::once(val),
                    DUMMY_SP,
                    should_be_named,
                    cb,
                )?;
            }
            Pat::Expr(expr) => {
                Self::assign_expr(realm, expr, value.next().unwrap_or(Value::Undefined), scope)?;
            }
            Pat::Invalid(i) => {
                return Err(Error::syn("Invalid pattern"));
            }
        }

        Ok(())
    }

    pub fn prop_name_to_value(
        realm: &mut Realm,
        prop: &PropName,
        scope: &mut Scope,
    ) -> Res<PropertyKey> {
        Ok(match prop {
            PropName::Ident(ident) => PropertyKey::String(YSString::from_ref(&ident.sym)),
            PropName::Str(str_) => {
                if let Some(s) = str_.value.as_str() {
                    PropertyKey::String(YSString::from_ref(s))
                } else {
                    let utf16_units: Vec<u16> = str_.value.to_ill_formed_utf16().collect();
                    PropertyKey::String(YSString::from_utf16(&utf16_units))
                }
            }
            PropName::Num(num) => PropertyKey::String(num.value.to_string().into()),
            PropName::Computed(expr) => {
                Self::run_expr(realm, &expr.expr, expr.span, scope)?.into_property_key(realm)?
            }
            PropName::BigInt(_) => todo!(),
        })
    }

    pub fn expr_should_be_named(expr: &Expr) -> bool {
        match expr {
            Expr::Fn(_) => true,
            Expr::Arrow(_) => true,
            Expr::Class(_) => true,
            Expr::Paren(paren) => Self::expr_should_be_named(&paren.expr),
            _ => false,
        }
    }
}

pub fn set_value_name(name: &str, value: &Value, realm: &mut Realm) -> Res {
    if name.is_empty() {
        return Ok(());
    }

    if let Value::Object(obj) = value {
        if let Some(f) = obj.downcast::<JSFunction>() {
            f.update_name(name)?;
        }

        if let Some(arrow) = obj.downcast::<ArrowFunction>() {
            arrow.define_property_attributes(
                "name".into(),
                YSString::from_ref(name).into(),
                realm,
            )?;
        }

        if let Some(class) = obj.downcast::<Class>() {
            class.update_name(name, realm)?;
        }

        if let Some(gen) = obj.downcast::<GeneratorFunction>() {
            gen.update_name(name, realm)?;
        }

        if let Some(gen) = obj.downcast::<AsyncGeneratorFunction>() {
            gen.update_name(name, realm)?;
        }
    }

    Ok(())
}
