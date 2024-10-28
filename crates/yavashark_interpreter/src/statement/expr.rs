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
use yavashark_env::{Realm, ControlFlow, RuntimeResult};

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
    pub fn run_expr_stmt(realm: &mut Realm, stmt: &ExprStmt, scope: &mut Scope) -> RuntimeResult {
        Self::run_expr(realm, &stmt.expr, stmt.span, scope)
    }
    pub fn run_expr(
        realm: &mut Realm,
        expr: &Expr,
        span: Span,
        scope: &mut Scope,
    ) -> RuntimeResult {
        match expr {
            Expr::This(stmt) => Self::run_this(realm, stmt, scope),
            Expr::Array(stmt) => Self::run_array(realm, stmt, scope),
            Expr::Object(stmt) => Self::run_object(realm, stmt, scope),
            Expr::Fn(stmt) => Self::run_fn(realm, stmt, scope),
            Expr::Unary(stmt) => Self::run_unary(realm, stmt, scope),
            Expr::Update(stmt) => Self::run_update(realm, stmt, scope),
            Expr::Bin(stmt) => Self::run_bin(realm, stmt, scope),
            Expr::Assign(stmt) => Self::run_assign(realm, stmt, scope),
            Expr::Member(stmt) => Self::run_member(realm, stmt, scope),
            Expr::SuperProp(stmt) => Self::run_super_prop(realm, stmt, scope),
            Expr::Cond(stmt) => Self::run_cond(realm, stmt, scope),
            Expr::Call(stmt) => Ok(Self::run_call(realm, stmt, scope)?),
            Expr::New(stmt) => Self::run_new(realm, stmt, scope),
            Expr::Seq(stmt) => Self::run_seq(realm, stmt, scope),
            Expr::Ident(stmt) => Self::run_ident(realm, stmt, scope),
            Expr::Lit(stmt) => Self::run_lit(realm, stmt),
            Expr::Tpl(stmt) => Self::run_tpl(realm, stmt, scope),
            Expr::TaggedTpl(stmt) => Self::run_tagged_tpl(realm, stmt, scope),
            Expr::Arrow(stmt) => Self::run_arrow(realm, stmt, scope),
            Expr::Class(stmt) => Self::run_class(realm, stmt, scope),
            Expr::Yield(stmt) => Self::run_yield(realm, stmt, scope),
            Expr::MetaProp(stmt) => Self::run_meta_prop(realm, stmt, scope),
            Expr::Await(stmt) => Self::run_await(realm, stmt, scope),
            Expr::Paren(stmt) => Self::run_paren(realm, stmt, scope),
            Expr::PrivateName(stmt) => Self::run_private_name(realm, stmt, scope),
            Expr::OptChain(stmt) => Self::run_opt_chain(realm, stmt, scope),
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
