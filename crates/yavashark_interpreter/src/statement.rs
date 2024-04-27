mod block;
mod debugger;
mod with;
mod r#return;
mod labeled;
mod r#continue;
mod r#if;
mod switch;
mod r#try;
mod r#while;
mod do_while;
mod r#for;
mod for_in;
mod for_of;
mod decl;
pub mod expr;
mod r#break;
mod throw;

use swc_ecma_ast::Stmt;
use yavashark_value::error::Error;
use yavashark_value::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {

    pub fn run_statement(&mut self, stmt: &Stmt, scope: &mut Scope) -> RuntimeResult {
        match stmt {
            Stmt::Block(block) => self.run_block(block, scope),
            Stmt::Empty(_) => Ok(Value::Undefined),
            Stmt::Debugger(d) => self.run_debugger(d, scope),
            Stmt::With(w) => self.run_with(w, scope),
            Stmt::Return(r) => self.run_return(r, scope),
            Stmt::Labeled(l) => self.run_labeled(l, scope),
            Stmt::Break(b) => self.run_break(b, scope),
            Stmt::Continue(c) => self.run_continue(c, scope),
            Stmt::If(i) => self.run_if(i, scope),
            Stmt::Switch(s) => self.run_switch(s, scope),
            Stmt::Throw(t) => self.run_throw(t, scope),
            Stmt::Try(t) => self.run_try(t, scope),
            Stmt::While(w) => self.run_while(w, scope),
            Stmt::DoWhile(d) => self.run_do_while(d, scope),
            Stmt::For(f) => self.run_for(f, scope),
            Stmt::ForIn(f) => self.run_for_in(f, scope),
            Stmt::ForOf(f) => self.run_for_of(f, scope),
            Stmt::Decl(d) => self.run_decl(d, scope).map(|_| Value::Undefined).map_err(|e| e.into()),
            Stmt::Expr(expr) => self.run_expr_stmt(expr, scope),
        }
    }
}
