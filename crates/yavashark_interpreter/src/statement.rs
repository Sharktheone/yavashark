use swc_common::Spanned;
use swc_ecma_ast::{Decl, ForHead, Stmt, VarDeclKind, VarDeclOrExpr};

use yavashark_env::{Realm, Res, RuntimeResult, Value, scope::Scope};

use crate::Interpreter;
use crate::location::get_location;

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

    pub fn run_statements(realm: &mut Realm, script: &[Stmt], scope: &mut Scope) -> RuntimeResult {
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

    fn hoist_statements(realm: &mut Realm, script: &[Stmt], scope: &mut Scope) -> Res<()> {
        for stmt in script {
            Self::hoist_stmt_impl::<false>(realm, stmt, scope)?;
        }

        Ok(())
    }

    fn hoist_globals(
        realm: &mut Realm,
        block: &swc_ecma_ast::BlockStmt,
        scope: &mut Scope,
    ) -> Res<()> {
        Self::hoist_global_stmts(realm, &block.stmts, scope)
    }

    fn hoist_global_stmts(realm: &mut Realm, stmts: &[Stmt], scope: &mut Scope) -> Res {
        for stmt in stmts {
            Self::hoist_stmt_impl::<true>(realm, stmt, scope)?;
        }

        Ok(())
    }

    fn hoist_stmt_impl<const GLOBAL: bool>(realm: &mut Realm, stmt: &Stmt, scope: &mut Scope) -> Res {
        match stmt {
            Stmt::Decl(decl) => {
                if GLOBAL {
                    Self::hoist_global_decl(realm, decl, scope)?;
                } else {
                    Self::hoist_decl(realm, decl, scope)?;
                }
            }
            Stmt::Block(block) => {
                Self::hoist_global_stmts(realm, &block.stmts, scope)?;
            }
            Stmt::If(i) => {
                Self::hoist_stmt_impl::<GLOBAL>(realm, &i.cons, scope)?;
                if let Some(alt) = &i.alt {
                    Self::hoist_stmt_impl::<GLOBAL>(realm, alt, scope)?;
                }
            }
            Stmt::Switch(s) => {
                for case in &s.cases {
                    Self::hoist_global_stmts(realm, &case.cons, scope)?;
                }
            }
            Stmt::Try(t) => {
                Self::hoist_global_stmts(realm, &t.block.stmts, scope)?;
                if let Some(handler) = &t.handler {
                    Self::hoist_global_stmts(realm, &handler.body.stmts, scope)?;
                }
                if let Some(finalizer) = &t.finalizer {
                    Self::hoist_global_stmts(realm, &finalizer.stmts, scope)?;
                }
            }
            Stmt::While(w) => {
                Self::hoist_stmt_impl::<GLOBAL>(realm, &w.body, scope)?;
            }
            Stmt::DoWhile(d) => {
                Self::hoist_stmt_impl::<GLOBAL>(realm, &d.body, scope)?;
            }
            Stmt::For(f) => {
                if let Some(VarDeclOrExpr::VarDecl(v)) = &f.init
                    && v.kind == VarDeclKind::Var
                {
                    Self::hoist_var(realm, v, scope)?;
                }
                Self::hoist_stmt_impl::<GLOBAL>(realm, &f.body, scope)?;
            }
            Stmt::ForIn(f) => {
                if let ForHead::VarDecl(v) = &f.left
                    && v.kind == VarDeclKind::Var
                {
                    Self::hoist_var(realm, v, scope)?;
                }
                Self::hoist_stmt_impl::<GLOBAL>(realm, &f.body, scope)?;
            }
            Stmt::ForOf(f) => {
                if let ForHead::VarDecl(v) = &f.left
                    && v.kind == VarDeclKind::Var
                {
                    Self::hoist_var(realm, v, scope)?;
                }
                Self::hoist_stmt_impl::<GLOBAL>(realm, &f.body, scope)?;
            }
            Stmt::With(w) => {
                Self::hoist_stmt_impl::<GLOBAL>(realm, &w.body, scope)?;
            }
            Stmt::Labeled(l) => {
                Self::hoist_stmt_impl::<GLOBAL>(realm, &l.body, scope)?;
            }
            _ => {}
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
            || matches!(self, Self::Block(block) if block.stmts.is_empty())
    }
}
