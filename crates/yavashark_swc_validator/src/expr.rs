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

impl Validator {
    pub fn validate_expr_stmt(expr: &ExprStmt) -> Result<(), String> {
        Self::validate_expr(&expr.expr)
    }

    pub fn validate_expr(expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::This(_) => Ok(()),
            Expr::Array(array) => Self::validate_array_expr(array),
            Expr::Object(object) => Self::validate_object_expr(object),
            Expr::Fn(func) => Self::validate_function_expr(func),
            Expr::Unary(unary) => Self::validate_unary_expr(unary),
            Expr::Update(update) => Self::validate_update_expr(update),
            Expr::Bin(bin) => Self::validate_binary_expr(bin),
            Expr::Assign(assign) => Self::validate_assign_expr(assign),
            Expr::Member(member) => Self::validate_member_expr(member),
            Expr::SuperProp(super_prop) => Self::validate_super_prop_expr(super_prop),
            Expr::Cond(cond) => Self::validate_cond_expr(cond),
            Expr::Call(call) => Self::validate_call_expr(call),
            Expr::New(new) => Self::validate_new_expr(new),
            Expr::Seq(seq) => Self::validate_seq_expr(seq),
            Expr::Ident(ident) => Self::validate_ident(ident),
            Expr::Lit(lit) => Self::validate_lit(lit),
            Expr::Tpl(tpl) => Self::validate_tpl_expr(tpl),
            Expr::TaggedTpl(tagged_tpl) => Self::validate_tagged_tpl_expr(tagged_tpl),
            Expr::Arrow(arrow) => Self::validate_arrow_expr(arrow),
            Expr::Class(class) => Self::validate_class_expr(class),
            Expr::Yield(yield_expr) => Self::validate_yield_expr(yield_expr),
            Expr::MetaProp(meta_prop) => Self::validate_meta_prop_expr(meta_prop),
            Expr::Await(await_expr) => Self::validate_await_expr(await_expr),
            Expr::Paren(paren) => Self::validate_expr(&paren.expr),
            Expr::PrivateName(private_name) => Self::validate_private_name_expr(private_name),
            Expr::OptChain(opt_chain) => Self::validate_opt_chain_expr(opt_chain),

            _ => Err(format!("Unsupported expression type: {expr:?}")),
        }
    }
}
