mod lit;
mod paren;

use crate::{Compiler, Res};
use anyhow::anyhow;
use swc_ecma_ast::Expr;
use yavashark_bytecode::data::{Acc, Data, DataType};
use yavashark_bytecode::jmp::Test;

impl Compiler {
    pub fn compile_test_expr(&mut self, test: &Expr) -> Res<Test> {
        let out = Some(Acc);

        //TODO: implement side effects (e.g if Array has a function call)
        #[allow(clippy::match_same_arms)]
        match test {
            Expr::This(_) => return Ok(Test::Never),
            Expr::Array(_) => return Ok(Test::Never),
            Expr::Object(_) => return Ok(Test::Never),
            Expr::Fn(_) => return Ok(Test::Never),
            Expr::Unary(u) => self.compile_unary(u, out)?,
            Expr::Update(u) => self.compile_update(u, out)?,
            Expr::Bin(b) => self.compile_bin(b, out)?,
            Expr::Assign(a) => self.compile_assign(a, out)?,
            Expr::Member(m) => self.compile_member(m, out)?,
            Expr::SuperProp(s) => self.compile_super_prop(s, out)?,
            Expr::Cond(c) => self.compile_cond(c, out)?,
            Expr::Call(c) => self.compile_call(c, out)?,
            Expr::New(n) => self.compile_new(n, out)?,
            Expr::Seq(s) => self.compile_seq(s, out)?,
            Expr::Ident(i) => return Ok(Test::Not(self.get_ident(i).data_type())),
            Expr::Lit(l) => return Ok(self.test_lit(l)),
            Expr::Tpl(t) => self.compile_tpl(t, out)?,
            Expr::TaggedTpl(t) => self.compile_tagged_tpl(t, out)?,
            Expr::Arrow(_) => return Ok(Test::Never),
            Expr::Class(_) => return Ok(Test::Never),
            Expr::Yield(y) => self.compile_yield(y, out)?,
            Expr::MetaProp(m) => self.compile_meta_prop(m, out)?,
            Expr::Await(a) => self.compile_await(a, out)?,
            Expr::Paren(p) => return self.test_paren(p),
            _ => return Err(anyhow!("Unsupported expression")),
        }

        Ok(Test::Not(DataType::Acc(Acc)))
    }
}
