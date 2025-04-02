mod block;
mod break_;
mod continue_;
mod debugger;
mod decl;
mod do_while;
mod expr;
mod for_;
mod for_in;
mod for_of;
mod if_;
mod labeled;
mod ret;
mod switch;
mod test;
mod throw;
mod try_;
mod while_;
mod with;

use crate::{Compiler, Res};
use swc_ecma_ast::Stmt;

impl Compiler {
    pub fn compile_stmt(&mut self, stmt: &Stmt) -> Res {
        match stmt {
            Stmt::Block(block) => self.compile_block(block),
            Stmt::Empty(_) => Ok(()),
            Stmt::Debugger(d) => self.compile_debugger(d),
            Stmt::With(w) => self.compile_with(w),
            Stmt::Return(r) => self.compile_return(r),
            Stmt::Labeled(l) => self.compile_labeled(l),
            Stmt::Break(b) => self.compile_break(b),
            Stmt::Continue(c) => self.compile_continue(c),
            Stmt::If(i) => self.compile_if(i),
            Stmt::Switch(s) => self.compile_switch(s),
            Stmt::Throw(t) => self.compile_throw(t),
            Stmt::Try(t) => self.compile_try(t),
            Stmt::While(w) => self.compile_while(w),
            Stmt::DoWhile(d) => self.compile_do_while(d),
            Stmt::For(f) => self.compile_for(f),
            Stmt::ForIn(f) => self.compile_for_in(f),
            Stmt::ForOf(f) => self.compile_for_of(f),
            Stmt::Decl(d) => self.compile_decl(d),
            Stmt::Expr(expr) => self.compile_expr_stmt(expr),
        }
    }

    pub fn compile_stmt_last(&mut self, stmt: &Stmt) -> Res {
        match stmt {
            Stmt::Block(block) => self.compile_block(block),
            Stmt::Empty(_) => Ok(()),
            Stmt::Debugger(d) => self.compile_debugger(d),
            Stmt::With(w) => self.compile_with(w),
            Stmt::Return(r) => self.compile_return(r),
            Stmt::Labeled(l) => self.compile_labeled(l),
            Stmt::Break(b) => self.compile_break(b),
            Stmt::Continue(c) => self.compile_continue(c),
            Stmt::If(i) => self.compile_if(i),
            Stmt::Switch(s) => self.compile_switch(s),
            Stmt::Throw(t) => self.compile_throw(t),
            Stmt::Try(t) => self.compile_try(t),
            Stmt::While(w) => self.compile_while(w),
            Stmt::DoWhile(d) => self.compile_do_while(d),
            Stmt::For(f) => self.compile_for(f),
            Stmt::ForIn(f) => self.compile_for_in(f),
            Stmt::ForOf(f) => self.compile_for_of(f),
            Stmt::Decl(d) => self.compile_decl(d),
            Stmt::Expr(expr) => self.compile_expr_stmt_last(expr),
        }
    }

    pub fn compile_stmts(&mut self, script: &[Stmt]) -> Res {
        for (idx, stmt) in script.iter().enumerate() {
            if idx == script.len() - 1 {
                self.compile_stmt_last(stmt)?;
            } else {
                self.compile_stmt(stmt)?;
            }
        }

        Ok(())
    }
}
