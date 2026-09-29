use super::function::FunctionKind;
use super::{Checker, FUNCTION, LEX, ScopeKind, VAR};
use crate::{Result, ValidationError};
use swc_common::Spanned;
use swc_ecma_ast::{Decl, Pat, VarDecl, VarDeclKind};

impl<'a> Checker<'a, '_> {
    pub(super) fn validate_decl(&mut self, decl: &'a Decl) -> Result<'a> {
        match decl {
            Decl::Var(var_decl) => self.validate_var(var_decl, false),
            Decl::Using(using_decl) => {
                if matches!(
                    self.scopes[self.scopes.len() - 1].kind,
                    ScopeKind::Script | ScopeKind::Switch
                ) {
                    return Err(ValidationError::new(
                        "Using declaration requires a block or module",
                        using_decl.span,
                    ));
                }

                if using_decl.is_await && !self.ctx.asynchronous && !self.ctx.module {
                    return Err(ValidationError::new(
                        "Await using outside async context",
                        using_decl.span,
                    ));
                }

                for decl in &using_decl.decls {
                    if Self::bound_contains(&decl.name, "let") {
                        return Err(ValidationError::new(
                            "Let is not a using binding name",
                            decl.span,
                        ));
                    }

                    self.validate_pat(&decl.name, LEX)?;

                    if let Some(e) = &decl.init {
                        self.validate_expr(e)?;
                    }
                }

                Ok(())
            }
            Decl::Fn(function) => {
                let binding_kind = if matches!(
                    self.scopes[self.scopes.len() - 1].kind,
                    ScopeKind::Block | ScopeKind::Switch | ScopeKind::Catch | ScopeKind::Module
                ) {
                    if !function.function.is_async && !function.function.is_generator {
                        LEX | FUNCTION
                    } else {
                        LEX
                    }
                } else {
                    VAR
                };
                self.declare_binding(&function.ident, binding_kind)?;
                self.validate_function(
                    &function.function,
                    Some(&function.ident),
                    FunctionKind::Declaration,
                )
            }
            Decl::Class(class) => {
                self.declare_binding(&class.ident, LEX)?;
                self.validate_class(&class.class, Some(&class.ident))
            }
            _ => Err(ValidationError::new(
                "Non-ECMAScript declaration",
                decl.span(),
            )),
        }
    }

    pub(super) fn validate_var(&mut self, var_decl: &'a VarDecl, loop_head: bool) -> Result<'a> {
        let kind = if var_decl.kind == VarDeclKind::Var {
            VAR
        } else {
            LEX
        };

        for d in &var_decl.decls {
            if kind == LEX && Self::bound_contains(&d.name, "let") {
                return Err(ValidationError::new(
                    "Let is not a lexical binding name",
                    d.span,
                ));
            }

            self.validate_pat(&d.name, kind)?;

            if let Some(e) = &d.init {
                self.validate_expr(e)?;
            } else if !loop_head
                && (var_decl.kind == VarDeclKind::Const || !matches!(d.name, Pat::Ident(_)))
            {
                return Err(ValidationError::new(
                    "Declaration requires an initializer",
                    d.span,
                ));
            }
        }

        Ok(())
    }
}
