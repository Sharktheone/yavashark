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

use swc_ecma_ast::{ModuleDecl, ModuleItem, Stmt};

pub struct Validator;

impl Validator {
    pub fn validate_statements(ast: &[Stmt]) -> Result<(), String> {
        for stmt in ast {
            Self::validate_statement(stmt)?;
        }
        Ok(())
    }

    pub fn validate_statement(stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block(block) => Self::validate_block(block),

            Stmt::Empty(empty) => Self::validate_empty(empty),
            Stmt::Debugger(debugger) => Self::validate_debugger(debugger),

            Stmt::With(with) => Self::validate_with(with),

            Stmt::Return(ret) => Self::validate_return(ret),

            Stmt::Labeled(labeled) => Self::validate_labeled(labeled),

            Stmt::Break(brk) => Self::validate_break(brk),

            Stmt::Continue(cnt) => Self::validate_continue(cnt),

            Stmt::If(i) => Self::validate_if(i),

            Stmt::Switch(switch) => Self::validate_switch(switch),

            Stmt::Throw(throw) => Self::validate_throw(throw),

            Stmt::Try(tryy) => Self::validate_try(tryy),

            Stmt::While(whle) => Self::validate_while(whle),

            Stmt::DoWhile(do_while) => Self::validate_do_while(do_while),

            Stmt::For(fr) => Self::validate_for(fr),

            Stmt::ForIn(for_in) => Self::validate_for_in(for_in),

            Stmt::ForOf(for_of) => Self::validate_for_of(for_of),

            Stmt::Decl(decl) => Self::validate_decl(decl),

            Stmt::Expr(expr) => Self::validate_expr_stmt(expr),
        }
    }

    pub fn validate_module_items(ast: &[ModuleItem]) -> Result<(), String> {
        for item in ast {
            match item {
                ModuleItem::Stmt(stmt) => Self::validate_statement(stmt)?,
                ModuleItem::ModuleDecl(item) => Self::validate_module_decl(item)?,
            }
        }
        Ok(())
    }

    pub fn validate_module_decl(_decl: &ModuleDecl) -> Result<(), String> {
        Ok(())
    }
}
