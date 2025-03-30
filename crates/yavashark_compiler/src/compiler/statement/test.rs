mod lit;

use anyhow::anyhow;
use swc_ecma_ast::Expr;
use yavashark_bytecode::data::{Acc, Data, DataType};
use yavashark_bytecode::jmp::Test;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_test_expr(&mut self, test: &Expr) -> Res<Test> {
        let out = Some(Acc);

        //TODO: implement side effects (e.g if Array has a function call)
        match test {
            Expr::This(_) => return Ok(Test::Unconditional),
            Expr::Array(_) => return Ok(Test::Unconditional),
            Expr::Object(o) => return Ok(Test::Unconditional),
            Expr::Fn(f) => return Ok(Test::Unconditional),
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
            Expr::Ident(i) => return Ok(Test::Cond(self.get_ident(i).data_type())),
            Expr::Lit(l) => return Ok(self.test_lit(l)),
            Expr::Tpl(t) => self.compile_tpl(t, out),
            Expr::TaggedTpl(t) => self.compile_tagged_tpl(t, out),
            Expr::Arrow(a) => return Ok(Test::Unconditional),
            Expr::Class(c) => return Ok(Test::Unconditional),
            Expr::Yield(y) => self.compile_yield(y, out),
            Expr::MetaProp(m) => self.compile_meta_prop(m, out),
            Expr::Await(a) => self.compile_await(a, out),
            Expr::Paren(p) => self.compile_paren(p, out),
            _ => Err(anyhow!("Unsupported expression")),
        }
        
        
        Ok(Test::Cond(DataType::Acc(Acc)))
    }
}