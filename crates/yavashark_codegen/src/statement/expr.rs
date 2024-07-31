use anyhow::anyhow;
use swc_common::Span;
use swc_ecma_ast::{Expr, ExprStmt};

use crate::{ByteCodegen, Res};

mod array;
mod arrow;
mod assign;
mod r#await;
mod bin;
mod call;
mod class;
mod cond;
mod r#fn;
mod ident;
mod lit;
mod member;
mod meta_prop;
mod new;
mod object;
mod opt_chain;
mod paren;
mod private_name;
mod seq;
mod super_prop;
mod tagged_tpl;
mod this;
mod tpl;
mod unary;
mod update;
mod r#yield;

impl ByteCodegen {
    pub fn compile_expr_stmt(&mut self, stmt: &ExprStmt) -> Res {
        self.compile_expr(&stmt.expr, stmt.span)
    }

    pub fn compile_expr(&mut self, expr: &Expr, span: Span) -> Res {
        match expr {
            Expr::This(stmt) => self.compile_this(stmt),
            Expr::Array(stmt) => self.compile_array(stmt),
            Expr::Object(stmt) => self.compile_object(stmt),
            Expr::Fn(stmt) => self.compile_fn(stmt),
            Expr::Unary(stmt) => self.compile_unary(stmt),
            Expr::Update(stmt) => self.compile_update(stmt),
            Expr::Bin(stmt) => self.compile_bin(stmt),
            Expr::Assign(stmt) => self.compile_assign(stmt),
            Expr::Member(stmt) => self.compile_member(stmt),
            Expr::SuperProp(stmt) => self.compile_super_prop(stmt),
            Expr::Cond(stmt) => self.compile_cond(stmt),
            Expr::Call(stmt) => Ok(self.compile_call(stmt)?),
            Expr::New(stmt) => self.compile_new(stmt),
            Expr::Seq(stmt) => self.compile_seq(stmt),
            Expr::Ident(stmt) => self.compile_ident(stmt),
            Expr::Lit(stmt) => self.compile_lit(stmt),
            Expr::Tpl(stmt) => self.compile_tpl(stmt),
            Expr::TaggedTpl(stmt) => self.compile_tagged_tpl(stmt),
            Expr::Arrow(stmt) => self.compile_arrow(stmt),
            Expr::Class(stmt) => self.compile_class(stmt),
            Expr::Yield(stmt) => self.compile_yield(stmt),
            Expr::MetaProp(stmt) => self.compile_meta_prop(stmt),
            Expr::Await(stmt) => self.compile_await(stmt),
            Expr::Paren(stmt) => self.compile_paren(stmt),
            Expr::PrivateName(stmt) => self.compile_private_name(stmt),
            Expr::OptChain(stmt) => self.compile_opt_chain(stmt),
            Expr::Invalid(stmt) => Err(anyhow!("{:?}: Invalid expression.", stmt.span)),
            _ => Err(anyhow!("{span:?}: TS and JSX are not supported.")),
        }?;

        Ok(())
    }
}
