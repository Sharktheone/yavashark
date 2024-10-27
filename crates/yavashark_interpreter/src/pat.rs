use swc_common::{Span, DUMMY_SP};
use swc_ecma_ast::{ObjectPatProp, Pat, PropName};

use yavashark_env::array::Array;
use yavashark_env::scope::Scope;
use yavashark_env::value::Obj;
use yavashark_env::{Context, Error, Object, Res, Value, ValueResult};

use crate::Interpreter;

impl Interpreter {
    pub fn run_pat(realm: &mut Realm, stmt: &Pat, scope: &mut Scope, value: Value) -> Res {
        Self::run_pat_internal(ctx, stmt, scope, value, false, DUMMY_SP)
    }

    #[allow(clippy::missing_panics_doc)] //Again, cannot panic in the real world
    pub fn run_pat_internal(
        realm: &mut Realm,
        stmt: &Pat,
        scope: &mut Scope,
        value: Value,
        for_in_of: bool,
        span: Span,
    ) -> Res {
        match stmt {
            Pat::Ident(id) => {
                scope.declare_var(id.sym.to_string(), value);
            }
            Pat::Array(arr) => {
                let mut iter = value.iter_no_ctx(ctx)?;

                let mut assert_last = false;

                for elem in &arr.elems {
                    if assert_last {
                        return Err(Error::syn("Rest element must be last element"));
                    }

                    let next = iter.next(ctx)?.unwrap_or(Value::Undefined);

                    if matches!(elem, Some(Pat::Rest(_))) {
                        #[allow(clippy::unwrap_used)]
                        let rest = elem.as_ref().unwrap(); // Safe to unwrap because of the match above

                        let mut elems = Vec::new();

                        while let Some(res) = iter.next(ctx)? {
                            elems.push(res);
                        }

                        let elems = Array::with_elements(ctx, elems)?.into_value();

                        Self::run_pat(ctx, rest, scope, elems)?;
                        let assert_last = true;
                    }

                    if let Some(elem) = elem {
                        Self::run_pat(ctx, elem, scope, next)?;
                    }
                }
            }
            Pat::Rest(rest) => {
                Self::run_pat(ctx, &rest.arg, scope, value)?;
            }
            Pat::Object(obj) => {
                let mut rest_not_props = Vec::with_capacity(obj.props.len());

                for prop in &obj.props {
                    match prop {
                        ObjectPatProp::KeyValue(kv) => {
                            let key = Self::prop_name_to_value(ctx, &kv.key, scope)?;
                            let value = value.get_property(&key, ctx).unwrap_or(Value::Undefined);

                            Self::run_pat(ctx, &kv.value, scope, value)?;
                            rest_not_props.push(key);
                        }
                        ObjectPatProp::Assign(assign) => {
                            let key = assign.key.sym.to_string();
                            let mut value = value
                                .get_property(&key.clone().into(), ctx)
                                .unwrap_or(Value::Undefined);

                            if let Some(val_expr) = &assign.value {
                                if value.is_nullish() {
                                    value = Self::run_expr(ctx, val_expr, assign.span, scope)?;
                                }
                            }

                            scope.declare_var(key.clone(), value);
                            rest_not_props.push(key.into());
                        }
                        ObjectPatProp::Rest(rest) => {
                            let mut rest_props = Vec::new();

                            for (name, value) in value.properties()? {
                                if !rest_not_props.contains(&name) {
                                    rest_props.push((name, value));
                                }
                            }

                            let rest_obj = Object::from_values(rest_props, ctx).into_value();

                            Self::run_pat(ctx, &rest.arg, scope, rest_obj)?;
                        }
                    }
                }
            }
            Pat::Assign(assign) => {
                let value = Self::run_expr(ctx, &assign.right, assign.span, scope)?;

                Self::run_pat(ctx, &assign.left, scope, value)?;
            }
            Pat::Expr(expr) => {
                if !for_in_of {
                    return Err(Error::syn("Invalid pattern"));
                }

                Self::run_expr(ctx, expr, span, scope)?;
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
            PropName::Computed(expr) => Self::run_expr(ctx, &expr.expr, expr.span, scope)?,
            PropName::BigInt(_) => todo!(),
        })
    }
}
