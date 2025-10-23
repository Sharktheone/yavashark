use swc_common::Spanned;
use swc_ecma_ast::{Decl, Stmt};

use yavashark_env::{scope::Scope, Realm, Res, RuntimeResult, Value};

use crate::location::get_location;
use crate::Interpreter;

mod block;
mod r#break;
mod r#continue;
mod debugger;
pub mod decl;
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
        let res = match stmt {
            Stmt::Block(block) => Self::run_block(realm, block, scope),
            Stmt::Empty(_) => Ok(Value::Undefined),
            Stmt::Debugger(d) => Self::run_debugger(realm, d, scope),
            Stmt::With(w) => Self::run_with(realm, w, scope),
            Stmt::Return(r) => Self::run_return(realm, r, scope),
            Stmt::Labeled(l) => Self::run_labeled(realm, l, scope),
            Stmt::Break(b) => Self::run_break(realm, b, scope),
            Stmt::Continue(c) => Self::run_continue(realm, c, scope),
            Stmt::If(i) => Self::run_if(realm, i, scope),
            Stmt::Switch(s) => Self::run_switch(realm, s, scope),
            Stmt::Throw(t) => Self::run_throw(realm, t, scope),
            Stmt::Try(t) => Self::run_try(realm, t, scope),
            Stmt::While(w) => Self::run_while(realm, w, scope),
            Stmt::DoWhile(d) => Self::run_do_while(realm, d, scope),
            Stmt::For(f) => Self::run_for(realm, f, scope),
            Stmt::ForIn(f) => Self::run_for_in(realm, f, scope),
            Stmt::ForOf(f) => Self::run_for_of(realm, f, scope),
            Stmt::Decl(d) => Self::run_decl(realm, d, scope)
                .map(|()| Value::Undefined)
                .map_err(std::convert::Into::into),
            Stmt::Expr(expr) => Self::run_expr_stmt(realm, expr, scope),
        };

        scope.set_no_label()?;

        res.map_err(|mut e| {
            e.attach_location(get_location(stmt.span(), scope));

            e
        })
    }

    pub fn run_statements(
        realm: &mut Realm,
        script: &Vec<Stmt>,
        scope: &mut Scope,
    ) -> RuntimeResult {
        Self::hoist_statements(realm, script, scope)?;

        let mut last_value = Value::Undefined;
        for stmt in script {
            if stmt.skip_statements() {
                continue;
            }
            let x = Self::run_statement(realm, stmt, scope);

            last_value = x?;
        }

        Ok(last_value)
    }

    fn hoist_statements(realm: &mut Realm, script: &Vec<Stmt>, scope: &mut Scope) -> Res<()> {
        for stmt in script {
            match stmt {
                Stmt::Decl(decl) => {
                    Self::hoist_decl(realm, decl, scope)?;
                }
                Stmt::Block(block) => {
                    Self::hoist_globals(realm, block, scope)?;
                }

                _ => {}
            }
        }

        Ok(())
    }

    fn hoist_globals(
        realm: &mut Realm,
        block: &swc_ecma_ast::BlockStmt,
        scope: &mut Scope,
    ) -> Res<()> {
        for stmt in &block.stmts {
            match stmt {
                Stmt::Decl(decl) => {
                    Self::hoist_global_decl(realm, decl, scope)?;
                }
                Stmt::Block(inner_block) => {
                    Self::hoist_globals(realm, inner_block, scope)?;
                }

                _ => {}
            }
        }

        Ok(())
    }
}

trait IsHoistable {
    fn skip_statements(&self) -> bool;
}

impl IsHoistable for Stmt {
    fn skip_statements(&self) -> bool {
        matches!(self, Self::Decl(Decl::Fn(_)) | Self::Empty(_))
    }
}
