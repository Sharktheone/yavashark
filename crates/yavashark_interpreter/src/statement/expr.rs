use swc_common::Span;
use swc_ecma_ast::{Expr, ExprStmt};

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
use yavashark_value::error::Error;
use crate::Value;

use crate::context::Context;
use crate::{ControlFlow, RuntimeResult};
use crate::scope::Scope;

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

impl Context {
    pub fn run_expr_stmt(&mut self, stmt: &ExprStmt, scope: &mut Scope) -> RuntimeResult {
        self.run_expr(&stmt.expr, stmt.span, scope)
    }
    pub fn run_expr(&mut self, expr: &Expr, span: Span, scope: &mut Scope) -> RuntimeResult {
        match expr {
            Expr::This(stmt) => { self.run_this(stmt, scope) }
            Expr::Array(stmt) => { self.run_array(stmt, scope) }
            Expr::Object(stmt) => { self.run_object(stmt, scope) }
            Expr::Fn(stmt) => { self.run_fn(stmt, scope) }
            Expr::Unary(stmt) => { self.run_unary(stmt, scope) }
            Expr::Update(stmt) => { self.run_update(stmt, scope) }
            Expr::Bin(stmt) => { self.run_bin(stmt, scope) }
            Expr::Assign(stmt) => { self.run_assign(stmt, scope) }
            Expr::Member(stmt) => { self.run_member(stmt, scope) }
            Expr::SuperProp(stmt) => { self.run_super_prop(stmt, scope) }
            Expr::Cond(stmt) => { self.run_cond(stmt, scope) }
            Expr::Call(stmt) => { self.run_call(stmt, scope) }
            Expr::New(stmt) => { self.run_new(stmt, scope) }
            Expr::Seq(stmt) => { self.run_seq(stmt, scope) }
            Expr::Ident(stmt) => { self.run_ident(stmt, scope) }
            Expr::Lit(stmt) => { self.run_lit(stmt) }
            Expr::Tpl(stmt) => { self.run_tpl(stmt, scope) }
            Expr::TaggedTpl(stmt) => { self.run_tagged_tpl(stmt, scope) }
            Expr::Arrow(stmt) => { self.run_arrow(stmt, scope) }
            Expr::Class(stmt) => { self.run_class(stmt, scope) }
            Expr::Yield(stmt) => { self.run_yield(stmt, scope) }
            Expr::MetaProp(stmt) => { self.run_meta_prop(stmt, scope) }
            Expr::Await(stmt) => { self.run_await(stmt, scope) }
            Expr::Paren(stmt) => { self.run_paren(stmt, scope) }
            Expr::PrivateName(stmt) => { self.run_private_name(stmt, scope) }
            Expr::OptChain(stmt) => { self.run_opt_chain(stmt, scope) }
            Expr::Invalid(stmt) => Err(ControlFlow::error(format!("{:?}: Invalid expression.", stmt.span))),
            _ => Err(ControlFlow::error(format!("{:?}: TS and JSX are not supported.", span))),
        }
    }
}
