mod block;
mod break_;
mod continue_;
mod debugger;
mod decl;
mod do_while;
mod empty;
mod expr;
mod for_;
mod for_in;
mod for_of;
mod if_;
mod labeled;
mod pat;
mod prop;
mod ret;
mod switch;
mod throw;
mod try_;
mod whle;
mod with;
mod utils;

use std::collections::HashSet;
use swc_ecma_ast::{ModuleDecl, ModuleItem, Stmt};
use crate::utils::FunctionContext;

#[derive(Default)]
pub struct Validator<'a> {
    function_ctx: Option<FunctionContext>,
    private_names: Vec<HashSet<&'a str>>
}

impl<'a> Validator<'a> {
    pub fn new() -> Self {
        Self::default()
    }


    pub fn validate_statements(&mut self, ast: &'a [Stmt]) -> Result<(), String> {
        for stmt in ast {
            self.validate_statement(stmt)?;
        }
        Ok(())
    }

    pub fn validate_statement(&mut self, stmt: &'a Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block(block) => self.validate_block(block),

            Stmt::Empty(empty) => self.validate_empty(empty),
            Stmt::Debugger(debugger) => self.validate_debugger(debugger),

            Stmt::With(with) => self.validate_with(with),

            Stmt::Return(ret) => self.validate_return(ret),

            Stmt::Labeled(labeled) => self.validate_labeled(labeled),

            Stmt::Break(brk) => self.validate_break(brk),

            Stmt::Continue(cnt) => self.validate_continue(cnt),

            Stmt::If(i) => self.validate_if(i),

            Stmt::Switch(switch) => self.validate_switch(switch),

            Stmt::Throw(throw) => self.validate_throw(throw),

            Stmt::Try(tryy) => self.validate_try(tryy),

            Stmt::While(whle) => self.validate_while(whle),

            Stmt::DoWhile(do_while) => self.validate_do_while(do_while),

            Stmt::For(fr) => self.validate_for(fr),

            Stmt::ForIn(for_in) => self.validate_for_in(for_in),

            Stmt::ForOf(for_of) => self.validate_for_of(for_of),

            Stmt::Decl(decl) => self.validate_decl(decl),

            Stmt::Expr(expr) => self.validate_expr_stmt(expr),
        }
    }

    pub fn validate_module_items(&mut self, ast: &'a [ModuleItem]) -> Result<(), String> {
        for item in ast {
            match item {
                ModuleItem::Stmt(stmt) => self.validate_statement(stmt)?,
                ModuleItem::ModuleDecl(item) => self.validate_module_decl(item)?,
            }
        }
        Ok(())
    }

    pub fn validate_module_decl(&mut self, _decl: &ModuleDecl) -> Result<(), String> {
        Ok(())
    }
}
