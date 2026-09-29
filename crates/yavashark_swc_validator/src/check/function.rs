use super::{Checker, Context, PARAM, ScopeKind};
use crate::{Result, ValidationError};
use swc_ecma_ast::{ArrowExpr, ArrowFunctionBody, Function, Ident, MethodKind, Pat};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum FunctionKind {
    Declaration,
    Expression,
    Method,
}

impl<'a> Checker<'a, '_> {
    pub(super) fn validate_function(
        &mut self,
        function: &'a Function,
        name: Option<&'a Ident>,
        kind: FunctionKind,
    ) -> Result<'a> {
        let previous_context = self.ctx;
        let has_strict_directive = function
            .body
            .as_ref()
            .is_some_and(|b| Self::strict_directive(&b.stmts));
        let simple_parameters = function
            .params
            .iter()
            .all(|p| matches!(p.pat, Pat::Ident(_)));

        if has_strict_directive && !simple_parameters {
            return Err(ValidationError::new(
                "Strict directive with non-simple parameters",
                function.span,
            ));
        }

        self.ctx = Context {
            strict: previous_context.strict | has_strict_directive,
            module: previous_context.module,
            function: true,
            asynchronous: function.is_async,
            generator: function.is_generator,
            super_prop: kind == FunctionKind::Method,
            new_target: true,
            label_base: self.labels.len(),
            ..Context::default()
        };

        if let Some(n) = name {
            if kind == FunctionKind::Expression {
                self.validate_binding(n)?;
            } else if self.ctx.strict
                && (matches!(n.sym.as_str(), "eval" | "arguments" | "yield")
                    || Self::strict_reserved(n.sym.as_str()))
            {
                return Err(ValidationError::new("Invalid strict function name", n.span));
            }
        }

        let capacity_hint = function.params.len().min(256)
            + function
                .body
                .as_ref()
                .map_or(0, |body| Self::direct_binding_count(&body.stmts));
        self.push_scope_with_capacity(ScopeKind::Function, capacity_hint);
        self.ctx.parameters = true;
        let unique_parameters = self.ctx.strict
            || !simple_parameters
            || kind == FunctionKind::Method
            || function.is_async
            || function.is_generator;

        for p in &function.params {
            self.validate_parameter(&p.pat, unique_parameters)?;
        }

        self.ctx.parameters = false;

        if let Some(b) = &function.body {
            self.validate_statements(&b.stmts)?;
        }

        self.scopes.pop();
        self.ctx = previous_context;

        Ok(())
    }

    pub(super) fn validate_parameter(
        &mut self,
        pat: &'a Pat,
        unique_parameters: bool,
    ) -> Result<'a> {
        self.ctx.unique_parameters = unique_parameters;
        self.validate_pat(pat, PARAM)
    }

    pub(super) fn validate_arrow(&mut self, arrow: &'a ArrowExpr) -> Result<'a> {
        let previous_context = self.ctx;
        let has_strict_directive = matches!(&*arrow.body,ArrowFunctionBody::FunctionBody(b) if Self::strict_directive(&b.stmts));

        if has_strict_directive && !arrow.params.iter().all(|p| matches!(p, Pat::Ident(_))) {
            return Err(ValidationError::new(
                "Strict directive with non-simple parameters",
                arrow.span,
            ));
        }

        self.ctx = Context {
            strict: previous_context.strict | has_strict_directive,
            function: true,
            asynchronous: arrow.is_async,
            generator: false,
            parameters: true,
            loops: 0,
            switches: 0,
            label_base: self.labels.len(),
            static_block: false,
            ..previous_context
        };
        self.push_scope(ScopeKind::Function);

        for p in &arrow.params {
            self.validate_parameter(p, true)?;
        }

        self.ctx.parameters = false;

        match &*arrow.body {
            ArrowFunctionBody::FunctionBody(b) => self.validate_statements(&b.stmts)?,
            ArrowFunctionBody::Expr(e) => self.validate_expr(e)?,
        }

        self.scopes.pop();
        self.ctx = previous_context;

        Ok(())
    }

    pub(super) fn validate_method_arity(function: &Function, kind: MethodKind) -> Result<'a> {
        if (kind == MethodKind::Getter && !function.params.is_empty())
            || (kind == MethodKind::Setter
                && (function.params.len() != 1 || matches!(function.params[0].pat, Pat::Rest(_))))
        {
            return Err(ValidationError::new(
                "Invalid accessor parameter list",
                function.span,
            ));
        }

        Ok(())
    }
}
