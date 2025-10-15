mod array;
mod arrow;
mod assign;
mod await_;
mod binary;
mod call;
mod class;
mod cond;
mod function;
mod ident;
mod lit;
mod member;
mod meta_prop;
mod new;
mod object;
mod opt_chain;
mod private_name;
mod seq;
mod super_prop;
mod taged_tpl;
mod tpl;
mod unary;
mod update;
mod yield_;

use crate::Validator;
use swc_ecma_ast::{Expr, ExprStmt};

impl<'a> Validator<'a> {
    pub fn validate_expr_stmt(&mut self, expr: &'a ExprStmt) -> Result<(), String> {
        self.validate_expr(&expr.expr)
    }

    pub fn validate_expr(&mut self, expr: &'a Expr) -> Result<(), String> {
        match expr {
            Expr::This(_) => Ok(()),
            Expr::Array(array) => self.validate_array_expr(array),
            Expr::Object(object) => self.validate_object_expr(object),
            Expr::Fn(func) => self.validate_function_expr(func),
            Expr::Unary(unary) => self.validate_unary_expr(unary),
            Expr::Update(update) => self.validate_update_expr(update),
            Expr::Bin(bin) => self.validate_binary_expr(bin),
            Expr::Assign(assign) => self.validate_assign_expr(assign),
            Expr::Member(member) => self.validate_member_expr(member),
            Expr::SuperProp(super_prop) => self.validate_super_prop_expr(super_prop),
            Expr::Cond(cond) => self.validate_cond_expr(cond),
            Expr::Call(call) => self.validate_call_expr(call),
            Expr::New(new) => self.validate_new_expr(new),
            Expr::Seq(seq) => self.validate_seq_expr(seq),
            Expr::Ident(ident) => self.validate_ident(ident),
            Expr::Lit(lit) => self.validate_lit(lit),
            Expr::Tpl(tpl) => self.validate_tpl_expr(tpl),
            Expr::TaggedTpl(tagged_tpl) => self.validate_tagged_tpl_expr(tagged_tpl),
            Expr::Arrow(arrow) => self.validate_arrow_expr(arrow),
            Expr::Class(class) => self.validate_class_expr(class),
            Expr::Yield(yield_expr) => self.validate_yield_expr(yield_expr),
            Expr::MetaProp(meta_prop) => self.validate_meta_prop_expr(meta_prop),
            Expr::Await(await_expr) => self.validate_await_expr(await_expr),
            Expr::Paren(paren) => self.validate_expr(&paren.expr),
            Expr::PrivateName(private_name) => self.validate_private_name_expr(private_name),
            Expr::OptChain(opt_chain) => self.validate_opt_chain_expr(opt_chain),

            _ => Err(format!("Unsupported expression type: {expr:?}")),
        }
    }
}
