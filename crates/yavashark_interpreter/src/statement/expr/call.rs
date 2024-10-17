use swc_common::Span;
use swc_ecma_ast::{CallExpr, Callee, Expr, ExprOrSpread, MemberExpr};

use yavashark_env::scope::Scope;
use yavashark_env::{Context, ControlFlow, Error, RuntimeResult, Value, ValueResult};

use crate::Interpreter;

impl Interpreter {
    pub fn run_call(ctx: &mut Context, stmt: &CallExpr, scope: &mut Scope) -> ValueResult {
        let Callee::Expr(callee_expr) = &stmt.callee else {
            return Err(Error::ty_error("Unsupported callee".to_string()));
        };

        let (callee, this) = Self::run_call_expr(ctx, callee_expr, stmt.span, scope)?;

        let this = this.unwrap_or(scope.this()?);

        Self::run_call_on(ctx, &callee, this, &stmt.args, stmt.span, scope)
    }

    pub fn run_call_on(
        ctx: &mut Context,
        callee: &Value,
        this: Value,
        args: &[ExprOrSpread],
        span: Span,
        scope: &mut Scope,
    ) -> ValueResult {
        if let Value::Object(f) = callee.copy() {
            let args = args
                .iter()
                .map(|arg| Self::run_expr(ctx, &arg.expr, arg.spread.unwrap_or(span), scope))
                .collect::<Result<Vec<Value>, ControlFlow>>()?;

            f.call(ctx, args, this) //In strict mode, this is undefined
        } else {
            Err(Error::ty_error(format!("{callee} is not a function",)))
        }
    }
    
    #[allow(clippy::cognitive_complexity)]
    pub fn run_call_expr(
        ctx: &mut Context,
        expr: &Expr,
        span: Span,
        scope: &mut Scope,
    ) -> Result<(Value, Option<Value>), ControlFlow> {
        Ok((
            match expr {
                Expr::This(stmt) => Self::run_this(ctx, stmt, scope)?,
                Expr::Array(stmt) => Self::run_array(ctx, stmt, scope)?,
                Expr::Object(stmt) => Self::run_object(ctx, stmt, scope)?,
                Expr::Fn(stmt) => Self::run_fn(ctx, stmt, scope)?,
                Expr::Unary(stmt) => Self::run_unary(ctx, stmt, scope)?,
                Expr::Update(stmt) => Self::run_update(ctx, stmt, scope)?,
                Expr::Bin(stmt) => Self::run_bin(ctx, stmt, scope)?,
                Expr::Assign(stmt) => Self::run_assign(ctx, stmt, scope)?,
                Expr::Member(stmt) => {
                    let (val, par) = Self::run_call_member(ctx, stmt, scope)?;

                    return Ok((val, Some(par)));
                }
                Expr::SuperProp(stmt) => Self::run_super_prop(ctx, stmt, scope)?,
                Expr::Cond(stmt) => Self::run_cond(ctx, stmt, scope)?,
                Expr::Call(stmt) => Self::run_call(ctx, stmt, scope)?,
                Expr::New(stmt) => Self::run_new(ctx, stmt, scope)?,
                Expr::Seq(stmt) => Self::run_seq(ctx, stmt, scope)?,
                Expr::Ident(stmt) => Self::run_ident(ctx, stmt, scope)?,
                Expr::Lit(stmt) => Self::run_lit(ctx, stmt)?,
                Expr::Tpl(stmt) => Self::run_tpl(ctx, stmt, scope)?,
                Expr::TaggedTpl(stmt) => Self::run_tagged_tpl(ctx, stmt, scope)?,
                Expr::Arrow(stmt) => Self::run_arrow(ctx, stmt, scope)?,
                Expr::Class(stmt) => Self::run_class(ctx, stmt, scope)?,
                Expr::Yield(stmt) => Self::run_yield(ctx, stmt, scope)?,
                Expr::MetaProp(stmt) => Self::run_meta_prop(ctx, stmt, scope)?,
                Expr::Await(stmt) => Self::run_await(ctx, stmt, scope)?,
                Expr::Paren(stmt) => Self::run_paren(ctx, stmt, scope)?,
                Expr::PrivateName(stmt) => Self::run_private_name(ctx, stmt, scope)?,
                Expr::OptChain(stmt) => Self::run_opt_chain(ctx, stmt, scope)?,
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
        ctx: &mut Context,
        stmt: &MemberExpr,
        scope: &mut Scope,
    ) -> Result<(Value, Value), ControlFlow> {
        let (value, par) = Self::run_call_expr(ctx, &stmt.obj, stmt.span, scope)?;

        Ok((
            Self::run_member_on(ctx, value.copy(), &stmt.prop, stmt.span, scope)?,
            par.unwrap_or(value),
        ))
    }
}
