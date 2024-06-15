use swc_common::Spanned;
use swc_ecma_ast::{ObjectLit, Prop, PropName, PropOrSpread};

use yavashark_env::{Context, ControlFlow, Object, RuntimeResult, Value};
use yavashark_env::scope::Scope;

use crate::function::JSFunction;
use crate::Interpreter;

impl Interpreter {
    pub fn run_object(ctx: &mut Context, stmt: &ObjectLit, scope: &mut Scope) -> RuntimeResult {
        let mut obj = Object::new(ctx);

        for prop in &stmt.props {
            match prop {
                PropOrSpread::Spread(spread) => {
                    let expr = Self::run_expr(ctx, &spread.expr, spread.dot3_token, scope)?;

                    if let Ok(props) = expr.properties() {
                        for (name, value) in props {
                            obj.define_property(name, value);
                        }
                    }
                }

                PropOrSpread::Prop(prop) => {
                    match &**prop {
                        Prop::Shorthand(ident) => {
                            let name = ident.sym.to_string();
                            let value = scope.resolve(&name)?.ok_or(
                                ControlFlow::error_reference(format!("{name} is not defined")),
                            )?;

                            obj.define_property(name.into(), value);
                        }
                        Prop::KeyValue(kv) => {
                            let key = Self::run_prop_name(ctx, &kv.key, scope)?;

                            let value = Self::run_expr(ctx, &kv.value, prop.span(), scope)?;

                            obj.define_property(key, value);
                        }
                        Prop::Assign(assign) => {
                            let key = assign.key.sym.to_string();

                            let value = Self::run_expr(ctx, &assign.value, prop.span(), scope)?;

                            obj.define_property(key.into(), value);
                        }

                        Prop::Method(method) => {
                            let key = Self::run_prop_name(ctx, &method.key, scope)?;
                            let mut fn_scope = Scope::with_parent(scope)?;

                            fn_scope.state_set_function();

                            let function = JSFunction::new(
                                key.to_string(), // TODO, what should the name be here? (and wrong to_string function)
                                method.function.params.clone(),
                                method.function.body.clone(),
                                fn_scope,
                                ctx,
                            );

                            let value = function.into();

                            obj.define_property(key, value);
                        }

                        _ => todo!(),
                    }
                }
            }
        }

        Ok(Value::Object(obj))
    }

    pub fn run_prop_name(ctx: &mut Context, prop: &PropName, scope: &mut Scope) -> RuntimeResult {
        Ok(match prop {
            PropName::Ident(ident) => Value::String(ident.sym.to_string()),
            PropName::Str(str_) => Value::String(str_.value.to_string()),
            PropName::Num(num) => Value::Number(num.value),
            PropName::Computed(expr) => Self::run_expr(ctx, &expr.expr, expr.span, scope)?,
            PropName::BigInt(_) => todo!(),
        })
    }
}
