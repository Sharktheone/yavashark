use swc_common::{Span, Spanned};
use swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, MemberExpr};

use crate::location::get_location;
use crate::Interpreter;
use yavashark_env::scope::Scope;
use yavashark_env::utils::ValueIterator;
use yavashark_env::{ControlFlow, Error, Realm, Value, ValueResult};

impl Interpreter {
    pub fn run_call(realm: &mut Realm, stmt: &CallExpr, scope: &mut Scope) -> ValueResult {
        match &stmt.callee {
            Callee::Expr(callee_expr) => {
                let (callee, this) = Self::run_call_expr(realm, callee_expr, stmt.span, scope)?;

                let this = this.unwrap_or(scope.this()?);

                Self::run_call_on(realm, &callee, this, &stmt.args, stmt.span, scope)
            }

            Callee::Super(sup) => {
                let class = scope.this()?;

                let proto = class.prototype(realm)?;
                let sup = proto.prototype(realm)?;

                let constructor = sup.as_object()?.constructor()?;

                let constructor = constructor.resolve(proto.copy(), realm)?;

                let constructor = constructor.as_object()?;

                let mut values = Vec::with_capacity(stmt.args.len());

                for arg in &stmt.args {
                    let value = Self::run_expr(realm, &arg.expr, arg.span(), scope)?;

                    if arg.spread.is_some() {
                        let iter = ValueIterator::new(&value, realm)?;

                        while let Some(value) = iter.next(realm)? {
                            values.push(value);
                        }
                    } else {
                        values.push(value);
                    }
                }

                //TODO: we somehow need to run the constructor ON the super class
                constructor
                    .construct(realm, values) //In strict mode, this is undefined
                    .map_err(|mut e| {
                        e.attach_function_stack(constructor.name(), get_location(stmt.span, scope));

                        e
                    })
            }

            Callee::Import(import) => {
                todo!()
            }
        }
    }

    pub fn run_call_on(
        realm: &mut Realm,
        callee: &Value,
        this: Value,
        args: &[ExprOrSpread],
        span: Span,
        scope: &mut Scope,
    ) -> ValueResult {
        if let Value::Object(f) = callee.copy() {
            let mut values = Vec::with_capacity(args.len());

            for arg in args {
                let value = Self::run_expr(realm, &arg.expr, arg.spread.unwrap_or(span), scope)?;

                if arg.spread.is_some() {
                    let iter = ValueIterator::new(&value, realm)?;

                    while let Some(value) = iter.next(realm)? {
                        values.push(value);
                    }
                } else {
                    values.push(value);
                }
            }

            f.call(realm, values, this) //In strict mode, this is undefined
                .map_err(|mut e| {
                    e.attach_function_stack(f.name(), get_location(span, scope));

                    e
                })
        } else {
            Err(Error::ty_error(format!("{callee} is not a function",)))
        }
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn run_call_expr(
        realm: &mut Realm,
        expr: &Expr,
        span: Span,
        scope: &mut Scope,
    ) -> Result<(Value, Option<Value>), ControlFlow> {
        Ok((
            match expr {
                Expr::This(stmt) => Self::run_this(realm, stmt, scope)?,
                Expr::Array(stmt) => Self::run_array(realm, stmt, scope)?,
                Expr::Object(stmt) => Self::run_object(realm, stmt, scope)?,
                Expr::Fn(stmt) => Self::run_fn(realm, stmt, scope)?,
                Expr::Unary(stmt) => Self::run_unary(realm, stmt, scope)?,
                Expr::Update(stmt) => Self::run_update(realm, stmt, scope)?,
                Expr::Bin(stmt) => Self::run_bin(realm, stmt, scope)?,
                Expr::Assign(stmt) => Self::run_assign(realm, stmt, scope)?,
                Expr::Member(stmt) => {
                    let (val, par) = Self::run_call_member(realm, stmt, scope)?;

                    return Ok((val, Some(par)));
                }
                Expr::SuperProp(stmt) => Self::run_super_prop(realm, stmt, scope)?,
                Expr::Cond(stmt) => Self::run_cond(realm, stmt, scope)?,
                Expr::Call(stmt) => Self::run_call(realm, stmt, scope)?,
                Expr::New(stmt) => Self::run_new(realm, stmt, scope)?,
                Expr::Seq(stmt) => Self::run_seq(realm, stmt, scope)?,
                Expr::Ident(stmt) => Self::run_ident(realm, stmt, scope)?,
                Expr::Lit(stmt) => Self::run_lit(realm, stmt)?,
                Expr::Tpl(stmt) => Self::run_tpl(realm, stmt, scope)?,
                Expr::TaggedTpl(stmt) => Self::run_tagged_tpl(realm, stmt, scope)?,
                Expr::Arrow(stmt) => Self::run_arrow(realm, stmt, scope)?,
                Expr::Class(stmt) => Self::run_class(realm, stmt, scope)?,
                Expr::Yield(stmt) => Self::run_yield(realm, stmt, scope)?,
                Expr::MetaProp(stmt) => Self::run_meta_prop(realm, stmt, scope)?,
                Expr::Await(stmt) => Self::run_await(realm, stmt, scope)?,
                Expr::Paren(stmt) => Self::run_paren(realm, stmt, scope)?,
                Expr::PrivateName(stmt) => Self::run_private_name(realm, stmt, scope)?,
                Expr::OptChain(stmt) => Self::run_opt_chain(realm, stmt, scope)?,
                Expr::Invalid(stmt) => {
                    return Err(ControlFlow::error(format!(
                        "{:?}: Invalid expression.",
                        stmt.span
                    )))
                }
                _ => {
                    return Err(ControlFlow::error(format!(
                        "{span:?}: TS and JSX are not supported."
                    )))
                }
            },
            None,
        ))
    }

    pub fn run_call_member(
        realm: &mut Realm,
        stmt: &MemberExpr,
        scope: &mut Scope,
    ) -> Result<(Value, Value), ControlFlow> {
        let (value, par) = Self::run_call_expr(realm, &stmt.obj, stmt.span, scope)?;

        let (member, this) =
            Self::run_member_on_primitives(realm, value.copy(), &stmt.prop, stmt.span, scope)?;

        Ok((member, this.or(par).unwrap_or(value)))
    }
}
