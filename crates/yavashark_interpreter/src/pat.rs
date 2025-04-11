use std::iter;
use swc_common::{Span, DUMMY_SP};
use swc_ecma_ast::{ObjectPatProp, Pat, PropName};

use crate::Interpreter;
use yavashark_env::array::Array;
use yavashark_env::scope::Scope;
use yavashark_env::value::Obj;
use yavashark_env::{Error, Object, Realm, Res, Value, ValueResult};
use yavashark_value::IntoValue;

impl Interpreter {
    pub fn run_pat(
        realm: &mut Realm,
        stmt: &Pat,
        scope: &mut Scope,
        value: &mut impl Iterator<Item = Value>,
    ) -> Res {
        Self::run_pat_internal(realm, stmt, scope, value, DUMMY_SP)
    }

    #[allow(clippy::missing_panics_doc)] //Again, cannot panic in the real world
    pub fn run_pat_internal(
        realm: &mut Realm,
        stmt: &Pat,
        scope: &mut Scope,
        value: &mut impl Iterator<Item = Value>,
        span: Span,
    ) -> Res {
        match stmt {
            Pat::Ident(id) => {
                let value = value.next().unwrap_or(Value::Undefined);
                scope.declare_var(id.sym.to_string(), value);
            }
            Pat::Array(arr) => {
                let mut iter = value
                    .next()
                    .unwrap_or(Value::Undefined)
                    .iter_no_realm(realm)?;

                let mut assert_last = false;

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

                        Self::run_pat(realm, rest, scope, &mut elems.into_iter())?;
                        let assert_last = true;
                    }

                    let next = iter.next(realm)?.unwrap_or(Value::Undefined);

                    if let Some(elem) = elem {
                        Self::run_pat(realm, elem, scope, &mut iter::once(next))?;
                    }
                }
            }
            Pat::Rest(rest) => {
                let collect = value.collect::<Vec<_>>();

                let array = Array::with_elements(realm, collect)?.into_value();

                Self::run_pat(realm, &rest.arg, scope, &mut iter::once(array))?;
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
                            let value =
                                object.get_property(&key, realm).unwrap_or(Value::Undefined);

                            Self::run_pat(realm, &kv.value, scope, &mut iter::once(value))?;
                            rest_not_props.push(key);
                        }
                        ObjectPatProp::Assign(assign) => {
                            let key = assign.key.sym.to_string();
                            let mut value = object
                                .get_property(&key.clone().into(), realm)
                                .unwrap_or(Value::Undefined);

                            if let Some(val_expr) = &assign.value {
                                if value.is_nullish() {
                                    value = Self::run_expr(realm, val_expr, assign.span, scope)?;
                                }
                            }

                            scope.declare_var(key.clone(), value);
                            rest_not_props.push(key.into());
                        }
                        ObjectPatProp::Rest(rest) => {
                            let mut rest_props = Vec::new();

                            for (name, value) in object.properties()? {
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
                            )?;
                        }
                    }
                }
            }
            Pat::Assign(assign) => {
                let val = value.next().unwrap_or(Value::Undefined);

                let val = if val.is_truthy() {
                    val
                } else {
                    Self::run_expr(realm, &assign.right, assign.span, scope)?
                };

                Self::run_pat(realm, &assign.left, scope, &mut iter::once(val))?;
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
    ) -> ValueResult {
        Ok(match prop {
            PropName::Ident(ident) => Value::String(ident.sym.to_string()),
            PropName::Str(str_) => Value::String(str_.value.to_string()),
            PropName::Num(num) => Value::Number(num.value),
            PropName::Computed(expr) => Self::run_expr(realm, &expr.expr, expr.span, scope)?,
            PropName::BigInt(_) => todo!(),
        })
    }
}
