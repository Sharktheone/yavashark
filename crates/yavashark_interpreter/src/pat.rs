use swc_common::{Span, DUMMY_SP};
use swc_ecma_ast::{ObjectPatProp, Pat, PropName};

use yavashark_value::Obj;

use crate::context::Context;
use crate::object::array::Array;
use crate::object::Object;
use crate::scope::Scope;
use crate::{Error, Res, Value, ValueResult};

impl Context {
    pub fn run_pat(&mut self, stmt: &Pat, scope: &mut Scope, value: Value) -> Res {
        self.run_pat_internal(stmt, scope, value, false, DUMMY_SP)
    }

    pub fn run_pat_internal(
        &mut self,
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
                let mut iter = value.iter_no_ctx(self)?;

                let mut assert_last = false;

                for elem in &arr.elems {
                    if assert_last {
                        return Err(Error::syn("Rest element must be last element"));
                    }

                    let next = iter.next(self)?.unwrap_or(Value::Undefined);

                    if matches!(elem, Some(Pat::Rest(_))) {
                        let rest = elem.as_ref().unwrap(); // Safe to unwrap because of the match above

                        let mut elems = Vec::new();

                        while let Some(res) = iter.next(self)? {
                            elems.push(res);
                        }

                        let elems = Array::from(elems).into_value();

                        self.run_pat(rest, scope, elems)?;
                        let assert_last = true;
                    }

                    if let Some(elem) = elem {
                        self.run_pat(elem, scope, next)?;
                    }
                }
            }
            Pat::Rest(rest) => {
                self.run_pat(&rest.arg, scope, value)?;
            }
            Pat::Object(obj) => {
                let mut rest_not_props = Vec::with_capacity(obj.props.len());

                for prop in &obj.props {
                    match prop {
                        ObjectPatProp::KeyValue(kv) => {
                            let key = self.prop_name_to_value(&kv.key, scope)?;
                            let value = value.get_property(&key).unwrap_or(Value::Undefined);

                            self.run_pat(&kv.value, scope, value)?;
                            rest_not_props.push(key);
                        }
                        ObjectPatProp::Assign(assign) => {
                            let key = assign.key.sym.to_string();
                            let mut value = value
                                .get_property(&key.clone().into())
                                .unwrap_or(Value::Undefined);

                            if let Some(val_expr) = &assign.value {
                                if value.is_nullish() {
                                    value = self.run_expr(val_expr, assign.span, scope)?;
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

                            let rest_obj = Object::from_values(rest_props, self).into_value();

                            self.run_pat(&rest.arg, scope, rest_obj)?;
                        }
                    }
                }
            }
            Pat::Assign(assign) => {
                let value = self.run_expr(&assign.right, assign.span, scope)?;

                self.run_pat(&assign.left, scope, value)?;
            }
            Pat::Expr(expr) => {
                if !for_in_of {
                    return Err(Error::syn("Invalid pattern"));
                }

                self.run_expr(expr, span, scope)?;
            }
            Pat::Invalid(i) => {
                return Err(Error::syn("Invalid pattern"));
            }
        }

        Ok(())
    }

    pub fn prop_name_to_value(&mut self, prop: &PropName, scope: &mut Scope) -> ValueResult {
        Ok(match prop {
            PropName::Ident(ident) => Value::String(ident.sym.to_string()),
            PropName::Str(str_) => Value::String(str_.value.to_string()),
            PropName::Num(num) => Value::Number(num.value),
            PropName::Computed(expr) => self.run_expr(&expr.expr, expr.span, scope)?,
            _ => todo!(),
        })
    }
}
