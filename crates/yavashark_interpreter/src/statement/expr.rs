use swc_common::Span;
use swc_ecma_ast::{Expr, ExprStmt};

use crate::Interpreter;
pub use arrow::*;
pub use assign::*;
pub use bin::*;
pub use call::*;
pub use class::*;
pub use cond::*;
pub use ident::*;
pub use lit::*;
pub use member::*;
pub use meta_prop::*;
pub use new::*;
pub use object::*;
pub use opt_chain::*;
pub use paren::*;
pub use private_name::*;
pub use r#array::*;
pub use r#await::*;
pub use r#fn::*;
pub use r#yield::*;
pub use seq::*;
pub use super_prop::*;
pub use tagged_tpl::*;
pub use this::*;
pub use tpl::*;
pub use unary::*;
pub use update::*;
use yavashark_env::scope::Scope;
use yavashark_env::{Context, ControlFlow, RuntimeResult};

mod this;

mod r#array;

mod object;

mod r#fn;

mod unary;

mod update;

mod bin;

mod assign;

mod member;

mod super_prop;

mod opt_chain;

mod private_name;

mod paren;

mod r#await;

mod meta_prop;

mod class;

mod arrow;

mod tpl;

mod tagged_tpl;

mod seq;

mod new;

mod lit;

mod ident;

mod cond;

mod call;

mod r#yield;

impl Interpreter {
    pub fn run_expr_stmt(ctx: &mut Context, stmt: &ExprStmt, scope: &mut Scope) -> RuntimeResult {
        Self::run_expr(ctx, &stmt.expr, stmt.span, scope)
    }
    pub fn run_expr(
        ctx: &mut Context,
        expr: &Expr,
        span: Span,
        scope: &mut Scope,
    ) -> RuntimeResult {
        match expr {
            Expr::This(stmt) => Self::run_this(ctx, stmt, scope),
            Expr::Array(stmt) => Self::run_array(ctx, stmt, scope),
            Expr::Object(stmt) => Self::run_object(ctx, stmt, scope),
            Expr::Fn(stmt) => Self::run_fn(ctx, stmt, scope),
            Expr::Unary(stmt) => Self::run_unary(ctx, stmt, scope),
            Expr::Update(stmt) => Self::run_update(ctx, stmt, scope),
            Expr::Bin(stmt) => Self::run_bin(ctx, stmt, scope),
            Expr::Assign(stmt) => Self::run_assign(ctx, stmt, scope),
            Expr::Member(stmt) => Self::run_member(ctx, stmt, scope),
            Expr::SuperProp(stmt) => Self::run_super_prop(ctx, stmt, scope),
            Expr::Cond(stmt) => Self::run_cond(ctx, stmt, scope),
            Expr::Call(stmt) => Ok(Self::run_call(ctx, stmt, scope)?),
            Expr::New(stmt) => Self::run_new(ctx, stmt, scope),
            Expr::Seq(stmt) => Self::run_seq(ctx, stmt, scope),
            Expr::Ident(stmt) => Self::run_ident(ctx, stmt, scope),
            Expr::Lit(stmt) => Self::run_lit(ctx, stmt),
            Expr::Tpl(stmt) => Self::run_tpl(ctx, stmt, scope),
            Expr::TaggedTpl(stmt) => Self::run_tagged_tpl(ctx, stmt, scope),
            Expr::Arrow(stmt) => Self::run_arrow(ctx, stmt, scope),
            Expr::Class(stmt) => Self::run_class(ctx, stmt, scope),
            Expr::Yield(stmt) => Self::run_yield(ctx, stmt, scope),
            Expr::MetaProp(stmt) => Self::run_meta_prop(ctx, stmt, scope),
            Expr::Await(stmt) => Self::run_await(ctx, stmt, scope),
            Expr::Paren(stmt) => Self::run_paren(ctx, stmt, scope),
            Expr::PrivateName(stmt) => Self::run_private_name(ctx, stmt, scope),
            Expr::OptChain(stmt) => Self::run_opt_chain(ctx, stmt, scope),
            Expr::Invalid(stmt) => Err(ControlFlow::error(format!(
                "{:?}: Invalid expression.",
                stmt.span
            ))),
            _ => Err(ControlFlow::error(format!(
                "{span:?}: TS and JSX are not supported."
            ))),
        }
    }
}
