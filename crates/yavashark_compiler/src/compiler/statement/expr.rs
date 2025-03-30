mod ident;
mod lit;
mod this;
mod array;
mod object;
mod fn_;
mod unary;
mod update;
mod bin;
mod assign;
mod member;
mod super_prop;
mod cond;
mod call;
mod new;
mod seq;
mod tpl;
mod tagged_tpl;
mod arrow;
mod class;
mod yield_;
mod meta_prop;
mod await_;
mod paren;

use crate::{Compiler, Res};
use anyhow::anyhow;
use swc_ecma_ast::{Expr, ExprStmt};
use yavashark_bytecode::data::{Acc, Data, DataType, OutputData};
use yavashark_bytecode::instructions::Instruction;

#[must_use]
pub struct MoveOptimization {
    pub output: DataType,
    pub reject_instructions: Vec<Instruction>,
}

impl MoveOptimization {
    pub fn new(out: impl Data, reject_instructions: Vec<Instruction>) -> Self {
        Self {
            output: out.data_type(),
            reject_instructions,
        }
    }
    pub fn accept(self) -> DataType {
        self.output
    }
    
    pub fn reject(self, compiler: &mut Compiler) {
        compiler.instructions.extend(self.reject_instructions);
    }
}

impl Compiler {
    
    pub fn compile_expr_stmt(&mut self, expr: &ExprStmt) -> Res {
        self.compile_expr_no_out(&expr.expr)
    }
    
    pub fn compile_expr(&mut self, expr: &Expr, out: Option<impl OutputData>) -> Res<Option<MoveOptimization>> {
        match expr {
            Expr::This(this) => self.compile_this(this, out),
            Expr::Array(a) => return self.compile_array(a, out),
            Expr::Object(o) => return self.compile_object(o, out),
            Expr::Fn(f) => return self.compile_fn(f, out),
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
            Expr::Ident(i) => return Ok(self.compile_ident(i, out)),
            Expr::Lit(l) => return self.compile_lit(l, out),
            Expr::Tpl(t) => self.compile_tpl(t, out)?,
            Expr::TaggedTpl(t) => self.compile_tagged_tpl(t, out)?,
            Expr::Arrow(a) => self.compile_arrow(a, out)?,
            Expr::Class(c) => return self.compile_class(c, out),
            Expr::Yield(y) => self.compile_yield(y, out)?,
            Expr::MetaProp(m) => self.compile_meta_prop(m, out)?,
            Expr::Await(a) => self.compile_await(a, out)?,
            Expr::Paren(p) => return self.compile_paren(p, out),
            _ => return Err(anyhow!("Unsupported expression")),
        }

        Ok(None)
    }
    
    pub fn compile_expr_data(&mut self, expr: &Expr, out: Option<impl OutputData>) -> Res<DataType> {
        match self.compile_expr(expr, out)? {
            Some(optim) => {
                Ok(optim.output)
            }
            None => Ok(DataType::Acc(Acc)),
        }
    }
    
    pub fn compile_expr_data_acc(&mut self, expr: &Expr) -> Res<DataType> {
        self.compile_expr_data(expr, Some(Acc))
    }
    
    pub fn compile_expr_no_out(&mut self, expr: &Expr) -> Res {
        self.compile_expr(expr, None::<Acc>)?;
        Ok(())
    }
}
