mod ident;
mod lit;
mod this;

use crate::{Compiler, Res};
use anyhow::anyhow;
use swc_ecma_ast::Expr;
use yavashark_bytecode::data::OutputData;

impl Compiler {
    pub fn compile_expr(&mut self, expr: &Expr, out: Option<impl OutputData>) -> Res {
        match expr {
            Expr::This(this) => self.compile_this(this, out),
            Expr::Array(a) => self.compile_array(a, out),
            Expr::Object(o) => self.compile_object(o, out),
            Expr::Fn(f) => self.compile_fn(f, out),
            Expr::Unary(u) => self.compile_unary(u, out),
            Expr::Update(u) => self.compile_update(u, out),
            Expr::Bin(b) => self.compile_bin(b, out),
            Expr::Assign(a) => self.compile_assign(a, out),
            Expr::Member(m) => self.compile_member(m, out),
            Expr::SuperProp(s) => self.compile_super_prop(s, out),
            Expr::Cond(c) => self.compile_cond(c, out),
            Expr::Call(c) => self.compile_call(c, out),
            Expr::New(n) => self.compile_new(n, out),
            Expr::Seq(s) => self.compile_seq(s, out),
            Expr::Ident(i) => self.compile_ident(i, out),
            Expr::Lit(l) => self.compile_lit(l, out),
            Expr::Tpl(t) => self.compile_tpl(t, out),
            Expr::TaggedTpl(t) => self.compile_tagged_tpl(t, out),
            Expr::Arrow(a) => self.compile_arrow(a, out),
            Expr::Class(c) => self.compile_class(c, out),
            Expr::Yield(y) => self.compile_yield(y, out),
            Expr::MetaProp(m) => self.compile_meta_prop(m, out),
            Expr::Await(a) => self.compile_await(a, out),
            Expr::Paren(p) => self.compile_paren(p, out),
            _ => Err(anyhow!("Unsupported expression")),
        }

        Ok(())
    }
}
