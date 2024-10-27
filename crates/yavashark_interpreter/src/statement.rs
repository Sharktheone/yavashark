use swc_ecma_ast::Stmt;

use yavashark_env::{scope::Scope, Context, RuntimeResult, Value};

use crate::Interpreter;

mod block;
mod r#break;
mod r#continue;
mod debugger;
mod decl;
mod do_while;
pub mod expr;
mod r#for;
mod for_in;
mod for_of;
mod r#if;
mod labeled;
mod r#return;
mod switch;
mod throw;
mod try_stmt;
mod r#while;
mod with;

impl Interpreter {
    pub fn run_statement(realm: &mut Realm, stmt: &Stmt, scope: &mut Scope) -> RuntimeResult {
        match stmt {
            Stmt::Block(block) => Self::run_block(ctx, block, scope),
            Stmt::Empty(_) => Ok(Value::Undefined),
            Stmt::Debugger(d) => Self::run_debugger(ctx, d, scope),
            Stmt::With(w) => Self::run_with(ctx, w, scope),
            Stmt::Return(r) => Self::run_return(ctx, r, scope),
            Stmt::Labeled(l) => Self::run_labeled(ctx, l, scope),
            Stmt::Break(b) => Self::run_break(ctx, b, scope),
            Stmt::Continue(c) => Self::run_continue(ctx, c, scope),
            Stmt::If(i) => Self::run_if(ctx, i, scope),
            Stmt::Switch(s) => Self::run_switch(ctx, s, scope),
            Stmt::Throw(t) => Self::run_throw(ctx, t, scope),
            Stmt::Try(t) => Self::run_try(ctx, t, scope),
            Stmt::While(w) => Self::run_while(ctx, w, scope),
            Stmt::DoWhile(d) => Self::run_do_while(ctx, d, scope),
            Stmt::For(f) => Self::run_for(ctx, f, scope),
            Stmt::ForIn(f) => Self::run_for_in(ctx, f, scope),
            Stmt::ForOf(f) => Self::run_for_of(ctx, f, scope),
            Stmt::Decl(d) => Self::run_decl(ctx, d, scope)
                .map(|()| Value::Undefined)
                .map_err(std::convert::Into::into),
            Stmt::Expr(expr) => Self::run_expr_stmt(ctx, expr, scope),
        }
    }

    pub fn run_statements(
        realm: &mut Realm,
        script: &Vec<Stmt>,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let mut last_value = Value::Undefined;
        for stmt in script {
            last_value = Self::run_statement(ctx, stmt, scope)?;
        }

        Ok(last_value)
    }
}
