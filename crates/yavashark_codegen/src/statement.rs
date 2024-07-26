mod block;
mod debugger;
mod with;
mod r#return;
mod labeled;
mod r#break;
mod r#continue;
mod r#if;
mod switch;
mod throw;
mod r#try;
mod r#while;
mod do_while;
mod r#for;
mod for_in;
mod for_of;
mod decl;
mod expr;

use swc_ecma_ast::Stmt;
use crate::ByteCodegen;

impl ByteCodegen {
    pub fn compile_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(block) => self.compile_block(block),
            Stmt::Empty(_) => {},
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
            Stmt::Expr(expr) => self.compile_expr(expr),
        }
    }


    pub fn compile_statements(&mut self, script: &Vec<Stmt>) {
        for stmt in script {
            self.compile_statement(stmt);
        }
    }
}

