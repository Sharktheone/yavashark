use super::{CATCH, Checker, LEX, ScopeKind};
use crate::{Result, ValidationError};
use swc_common::Spanned;
use swc_ecma_ast::{Decl, ForHead, Pat, Stmt, VarDeclKind, VarDeclOrExpr};

impl<'a> Checker<'a, '_> {
    pub(crate) fn validate_script(&mut self, stmts: &'a [Stmt]) -> Result<'a> {
        self.ctx.strict |= Self::strict_directive(stmts);
        self.push_scope_with_capacity(ScopeKind::Script, Self::direct_binding_count(stmts));
        self.validate_statements(stmts)
    }

    pub(super) fn validate_statements(&mut self, stmts: &'a [Stmt]) -> Result<'a> {
        for stmt in stmts {
            self.validate_statement(stmt)?;
        }

        Ok(())
    }

    pub(super) fn validate_block(&mut self, stmts: &'a [Stmt]) -> Result<'a> {
        self.push_scope_with_capacity(ScopeKind::Block, Self::direct_binding_count(stmts));
        self.validate_statements(stmts)?;
        self.scopes.pop();

        Ok(())
    }

    pub(super) fn validate_statement(&mut self, stmt: &'a Stmt) -> Result<'a> {
        self.depth += 1;
        let result = if self.depth.is_multiple_of(16) {
            stacker::maybe_grow(256 * 1024, 2 * 1024 * 1024, || {
                self.validate_statement_inner(stmt)
            })
        } else {
            self.validate_statement_inner(stmt)
        };
        self.depth -= 1;
        result
    }

    pub(super) fn validate_statement_inner(&mut self, stmt: &'a Stmt) -> Result<'a> {
        match stmt {
            Stmt::Block(block) => self.validate_block(&block.stmts)?,
            Stmt::Expr(expression) => {
                if Self::starts_with_let_bracket(&expression.expr) {
                    return Err(ValidationError::new(
                        "Forbidden let bracket expression statement",
                        expression.span,
                    ));
                }

                self.validate_expr(&expression.expr)?;
            }
            Stmt::Decl(decl) => self.validate_decl(decl)?,
            Stmt::Return(ret) => {
                if !self.ctx.function || self.ctx.static_block {
                    return Err(ValidationError::new("Return outside function", ret.span));
                }

                if let Some(e) = &ret.arg {
                    self.validate_expr(e)?;
                }
            }
            Stmt::Throw(throw) => self.validate_expr(&throw.arg)?,
            Stmt::With(with) => {
                if self.ctx.strict {
                    return Err(ValidationError::new("With in strict code", with.span));
                }

                self.validate_expr(&with.obj)?;
                self.validate_single_statement(&with.body, false)?;
            }
            Stmt::If(if_stmt) => {
                self.validate_expr(&if_stmt.test)?;
                self.validate_if_arm(&if_stmt.cons)?;

                if let Some(s) = &if_stmt.alt {
                    self.validate_if_arm(s)?;
                }
            }
            Stmt::While(while_stmt) => {
                self.validate_expr(&while_stmt.test)?;
                self.ctx.loops += 1;
                self.validate_single_statement(&while_stmt.body, false)?;
                self.ctx.loops -= 1;
            }
            Stmt::DoWhile(do_while) => {
                self.ctx.loops += 1;
                self.validate_single_statement(&do_while.body, false)?;
                self.ctx.loops -= 1;
                self.validate_expr(&do_while.test)?;
            }
            Stmt::For(for_stmt) => {
                self.push_scope(ScopeKind::Block);

                if let Some(i) = &for_stmt.init {
                    match i {
                        VarDeclOrExpr::VarDecl(v) => self.validate_var(v, false)?,
                        VarDeclOrExpr::Expr(e) => self.validate_expr(e)?,
                    }
                }

                if let Some(e) = &for_stmt.test {
                    self.validate_expr(e)?;
                }

                if let Some(e) = &for_stmt.update {
                    self.validate_expr(e)?;
                }

                self.ctx.loops += 1;
                self.validate_single_statement(&for_stmt.body, false)?;
                self.ctx.loops -= 1;
                self.scopes.pop();
            }
            Stmt::ForIn(for_in) => {
                self.push_scope(ScopeKind::Block);
                self.validate_for_head(&for_in.left, true)?;
                self.validate_expr(&for_in.right)?;
                self.ctx.loops += 1;
                self.validate_single_statement(&for_in.body, false)?;
                self.ctx.loops -= 1;
                self.scopes.pop();
            }
            Stmt::ForOf(for_of) => {
                if for_of.is_await
                    && (!self.ctx.asynchronous && !self.ctx.module || self.ctx.static_block)
                {
                    return Err(ValidationError::new(
                        "For await outside async context",
                        for_of.span,
                    ));
                }

                self.push_scope(ScopeKind::Block);
                self.validate_for_head(&for_of.left, false)?;
                self.validate_expr(&for_of.right)?;
                self.ctx.loops += 1;
                self.validate_single_statement(&for_of.body, false)?;
                self.ctx.loops -= 1;
                self.scopes.pop();
            }
            Stmt::Switch(switch) => {
                self.validate_expr(&switch.discriminant)?;
                self.push_scope(ScopeKind::Switch);
                self.ctx.switches += 1;
                let mut default = false;

                for c in &switch.cases {
                    if let Some(e) = &c.test {
                        self.validate_expr(e)?;
                    } else {
                        if default {
                            return Err(ValidationError::new("Duplicate switch default", c.span));
                        }
                        default = true;
                    }

                    self.validate_statements(&c.cons)?;
                }

                self.ctx.switches -= 1;
                self.scopes.pop();
            }
            Stmt::Break(brk) => {
                if let Some(l) = &brk.label {
                    if !self.labels[self.ctx.label_base..]
                        .iter()
                        .any(|(n, _)| *n == l.sym.as_str())
                    {
                        return Err(ValidationError::new("Unknown break label", brk.span));
                    }
                } else if self.ctx.loops == 0 && self.ctx.switches == 0 {
                    return Err(ValidationError::new(
                        "Break outside loop or switch",
                        brk.span,
                    ));
                }
            }
            Stmt::Continue(continue_stmt) => {
                if let Some(l) = &continue_stmt.label {
                    if !self.labels[self.ctx.label_base..]
                        .iter()
                        .any(|(n, loop_)| *n == l.sym.as_str() && *loop_)
                    {
                        return Err(ValidationError::new(
                            "Continue requires an iteration label",
                            continue_stmt.span,
                        ));
                    }
                } else if self.ctx.loops == 0 {
                    return Err(ValidationError::new(
                        "Continue outside loop",
                        continue_stmt.span,
                    ));
                }
            }
            Stmt::Labeled(labeled) => {
                self.validate_ident(&labeled.label)?;

                if self.labels[self.ctx.label_base..]
                    .iter()
                    .any(|(n, _)| *n == labeled.label.sym.as_str())
                {
                    return Err(ValidationError::new("Duplicate label", labeled.span));
                }

                self.labels
                    .push((&labeled.label.sym, Self::iteration(&labeled.body)));
                self.validate_single_statement(&labeled.body, true)?;
                self.labels.pop();
            }
            Stmt::Try(try_stmt) => {
                self.validate_block(&try_stmt.block.stmts)?;

                if let Some(c) = &try_stmt.handler {
                    self.push_scope(ScopeKind::Catch);

                    if let Some(p) = &c.param {
                        self.validate_pat(
                            p,
                            if matches!(p, Pat::Ident(_)) {
                                LEX | CATCH
                            } else {
                                LEX
                            },
                        )?;
                    }

                    self.validate_statements(&c.body.stmts)?;
                    self.scopes.pop();
                }

                if let Some(b) = &try_stmt.finalizer {
                    self.validate_block(&b.stmts)?;
                }
            }
            Stmt::Empty(_) | Stmt::Debugger(_) => {}
        }

        Ok(())
    }

    pub(super) fn validate_if_arm(&mut self, stmt: &'a Stmt) -> Result<'a> {
        if matches!(stmt,Stmt::Labeled(l) if Self::labelled_function(&l.body)) {
            return Err(ValidationError::new(
                "Labelled function in if statement",
                stmt.span(),
            ));
        }

        if matches!(stmt, Stmt::Decl(Decl::Fn(function))
            if !self.ctx.strict && !function.function.is_async && !function.function.is_generator)
        {
            self.push_scope(ScopeKind::Block);
            self.validate_statement(stmt)?;
            self.scopes.pop();

            return Ok(());
        }

        if matches!(stmt, Stmt::Decl(Decl::Fn(function))
            if !self.ctx.strict && !function.function.is_async && !function.function.is_generator)
        {
            // Annex B treats an if-arm function as though it were in a block.
            self.push_scope(ScopeKind::Block);
            self.validate_statement(stmt)?;
            self.scopes.pop();

            return Ok(());
        }

        self.validate_single_statement(stmt, true)
    }

    pub(super) fn validate_single_statement(
        &mut self,
        stmt: &'a Stmt,
        allow_annex_b_function: bool,
    ) -> Result<'a> {
        match stmt {
            Stmt::Decl(Decl::Var(v)) if v.kind == VarDeclKind::Var => {}
            Stmt::Decl(Decl::Fn(f))
                if allow_annex_b_function
                    && !self.ctx.strict
                    && !f.function.is_async
                    && !f.function.is_generator => {}
            Stmt::Decl(_) => {
                return Err(ValidationError::new(
                    "Declaration in single-statement position",
                    stmt.span(),
                ));
            }
            Stmt::Labeled(labeled)
                if !allow_annex_b_function && Self::labelled_function(&labeled.body) =>
            {
                return Err(ValidationError::new(
                    "Labelled function in statement position",
                    stmt.span(),
                ));
            }
            _ => {}
        }

        self.validate_statement(stmt)
    }

    pub(super) fn validate_for_head(&mut self, head: &'a ForHead, is_for_in: bool) -> Result<'a> {
        match head {
            ForHead::VarDecl(v) => {
                if v.decls.len() != 1 {
                    return Err(ValidationError::new(
                        "Loop declaration must have one binding",
                        v.span,
                    ));
                }

                let d = &v.decls[0];

                if d.init.is_some()
                    && !(is_for_in
                        && !self.ctx.strict
                        && v.kind == VarDeclKind::Var
                        && matches!(d.name, Pat::Ident(_)))
                {
                    return Err(ValidationError::new(
                        "Loop binding cannot have initializer",
                        d.span,
                    ));
                }

                self.validate_var(v, true)?;
            }
            ForHead::Pat(p) => self.validate_pat(p, 0)?,
            ForHead::UsingDecl(u) => {
                if is_for_in || u.decls.len() != 1 || u.decls[0].init.is_some() {
                    return Err(ValidationError::new(
                        "Invalid using loop declaration",
                        u.span,
                    ));
                }

                for d in &u.decls {
                    if Self::bound_contains(&d.name, "let") {
                        return Err(ValidationError::new(
                            "Let is not a using binding name",
                            d.span,
                        ));
                    }

                    self.validate_pat(&d.name, LEX)?;
                }
            }
        }

        Ok(())
    }
}
