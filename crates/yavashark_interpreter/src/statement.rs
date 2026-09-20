use swc_common::Spanned;
use swc_ecma_ast::{Decl, ForHead, Pat, Stmt, VarDecl, VarDeclKind, VarDeclOrExpr};

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
        Self::hoist_stmts::<false>(realm, script, scope, &mut Vec::new())
    }

    fn hoist_stmts<const GLOBAL: bool>(
        realm: &mut Realm,
        stmts: &[Stmt],
        scope: &mut Scope,
        lexical_names: &mut Vec<String>,
    ) -> Res {
        let len = lexical_names.len();
        lexical_names.extend(stmts.iter().flat_map(lexical_bound_names));

        for stmt in stmts {
            Self::hoist_stmt_impl::<GLOBAL>(realm, stmt, scope, lexical_names)?;
        }
        lexical_names.truncate(len);
        Ok(())
    }

    fn hoist_loop_binding(
        realm: &mut Realm,
        var: &VarDecl,
        scope: &mut Scope,
        lexical_names: &mut Vec<String>,
    ) -> Res {
        if var.kind == VarDeclKind::Var {
            Self::hoist_var(realm, var, scope)?;
        } else {
            lexical_names.extend(
                var.decls
                    .iter()
                    .flat_map(|decl| decl::pat_idents(&decl.name)),
            );
        }
        Ok(())
    }

    fn hoist_stmt_impl<const GLOBAL: bool>(
        realm: &mut Realm,
        stmt: &Stmt,
        scope: &mut Scope,
        lexical_names: &mut Vec<String>,
    ) -> Res {
        let len = lexical_names.len();
        match stmt {
            Stmt::Decl(decl) => {
                if GLOBAL {
                    if !matches!(decl, Decl::Fn(f) if lexical_names.contains(&f.ident.sym.to_string()))
                    {
                        Self::hoist_global_decl(realm, decl, scope)?;
                    }
                } else {
                    Self::hoist_decl(realm, decl, scope)?;
                }
            }
            Stmt::Block(block) => {
                Self::hoist_stmts::<true>(realm, &block.stmts, scope, lexical_names)?;
            }
            Stmt::If(i) => {
                Self::hoist_stmt_impl::<true>(realm, &i.cons, scope, lexical_names)?;
                if let Some(alt) = &i.alt {
                    Self::hoist_stmt_impl::<true>(realm, alt, scope, lexical_names)?;
                }
            }
            Stmt::Switch(s) => {
                lexical_names.extend(
                    s.cases
                        .iter()
                        .flat_map(|case| &case.cons)
                        .flat_map(lexical_bound_names),
                );
                for case in &s.cases {
                    Self::hoist_stmts::<true>(realm, &case.cons, scope, lexical_names)?;
                }
            }
            Stmt::Try(t) => {
                Self::hoist_stmts::<true>(realm, &t.block.stmts, scope, lexical_names)?;
                if let Some(handler) = &t.handler {
                    if let Some(param) = &handler.param
                        && !matches!(param, Pat::Ident(_))
                    {
                        lexical_names.extend(decl::pat_idents(param));
                    }
                    Self::hoist_stmts::<true>(realm, &handler.body.stmts, scope, lexical_names)?;
                }
                lexical_names.truncate(len);
                if let Some(finalizer) = &t.finalizer {
                    Self::hoist_stmts::<true>(realm, &finalizer.stmts, scope, lexical_names)?;
                }
            }
            Stmt::While(w) => {
                Self::hoist_stmt_impl::<GLOBAL>(realm, &w.body, scope, lexical_names)?;
            }
            Stmt::DoWhile(d) => {
                Self::hoist_stmt_impl::<GLOBAL>(realm, &d.body, scope, lexical_names)?;
            }
            Stmt::For(f) => {
                if let Some(VarDeclOrExpr::VarDecl(v)) = &f.init {
                    Self::hoist_loop_binding(realm, v, scope, lexical_names)?;
                }
                Self::hoist_stmt_impl::<GLOBAL>(realm, &f.body, scope, lexical_names)?;
            }
            Stmt::ForIn(f) => {
                if let ForHead::VarDecl(v) = &f.left {
                    Self::hoist_loop_binding(realm, v, scope, lexical_names)?;
                }
                Self::hoist_stmt_impl::<GLOBAL>(realm, &f.body, scope, lexical_names)?;
            }
            Stmt::ForOf(f) => {
                if let ForHead::VarDecl(v) = &f.left {
                    Self::hoist_loop_binding(realm, v, scope, lexical_names)?;
                }
                Self::hoist_stmt_impl::<GLOBAL>(realm, &f.body, scope, lexical_names)?;
            }
            Stmt::With(w) => {
                Self::hoist_stmt_impl::<GLOBAL>(realm, &w.body, scope, lexical_names)?;
            }
            Stmt::Labeled(l) => {
                Self::hoist_stmt_impl::<GLOBAL>(realm, &l.body, scope, lexical_names)?;
            }
            _ => {}
        }

        lexical_names.truncate(len);
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

fn lexical_bound_names(stmt: &Stmt) -> Vec<String> {
    match stmt {
        Stmt::Decl(Decl::Var(v)) if v.kind != VarDeclKind::Var => v
            .decls
            .iter()
            .flat_map(|decl| decl::pat_idents(&decl.name))
            .collect(),
        Stmt::Decl(Decl::Class(c)) => vec![c.ident.sym.to_string()],
        _ => Vec::new(),
    }
}
