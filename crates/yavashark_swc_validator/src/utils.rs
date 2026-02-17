use std::collections::HashSet;

use crate::Validator;
use swc_ecma_ast::{BlockStmt, Expr, ExprStmt, Lit, Stmt};
use unicode_ident::{is_xid_continue, is_xid_start};

#[must_use]
pub struct PrivateNameScope;

#[derive(Clone, Default)]
pub struct FunctionContext {
    is_async: bool,
    is_generator: bool,
    await_restricted: bool,
    param_names: HashSet<String>,
    is_strict: bool,
    allow_super_property: bool,
    allow_super_call: bool,
}

#[must_use]
pub struct FunctionContextScope(Option<FunctionContext>);

#[must_use]
pub struct BlockScopeGuard;

#[must_use]
pub struct AwaitRestrictionGuard;

#[must_use]
pub struct SuperPropertyScopeGuard;

#[must_use]
pub struct SuperCallScopeGuard;

#[must_use]
pub struct RelaxedAwaitGuard;

impl PrivateNameScope {
    #[allow(clippy::unused_self)]
    pub fn exit(self, validator: &mut Validator<'_>) {
        validator.private_names.pop();
    }
}

impl FunctionContextScope {
    pub fn exit(mut self, validator: &mut Validator) {
        validator.function_ctx = self.0.take();
    }
}

impl BlockScopeGuard {
    pub fn exit(self, validator: &mut Validator<'_>) {
        validator.param_shadow_stack.pop();
    }
}

impl AwaitRestrictionGuard {
    pub const fn exit(self, validator: &mut Validator<'_>) {
        validator.await_restriction_depth = validator.await_restriction_depth.saturating_sub(1);
    }
}

impl SuperPropertyScopeGuard {
    pub const fn exit(self, validator: &mut Validator<'_>) {
        validator.super_property_scope = validator.super_property_scope.saturating_sub(1);
    }
}

impl SuperCallScopeGuard {
    pub const fn exit(self, validator: &mut Validator<'_>) {
        validator.super_call_scope = validator.super_call_scope.saturating_sub(1);
    }
}

impl RelaxedAwaitGuard {
    pub const fn exit(self, validator: &mut Validator<'_>) {
        validator.await_relax_depth = validator.await_relax_depth.saturating_sub(1);
    }
}

impl<'a> Validator<'a> {
    pub fn enter_private_name_scope(&mut self, names: HashSet<&'a str>) -> PrivateNameScope {
        self.private_names.push(names);

        PrivateNameScope
    }

    pub fn enter_function_context(
        &mut self,
        is_async: bool,
        is_generator: bool,
    ) -> FunctionContextScope {
        let await_restricted = is_async
            || self
                .function_ctx
                .as_ref()
                .is_some_and(|ctx| ctx.await_restricted);
        let (allow_super_property, allow_super_call) =
            self.function_ctx.as_ref().map_or((false, false), |ctx| {
                (ctx.allow_super_property, ctx.allow_super_call)
            });
        let inherited_strict = self
            .function_ctx
            .as_ref()
            .map_or(self.script_strict, |ctx| ctx.is_strict);

        let old = self.function_ctx.replace(FunctionContext {
            is_async,
            is_generator,
            await_restricted,
            param_names: HashSet::new(),
            is_strict: inherited_strict,
            allow_super_property,
            allow_super_call,
        });

        FunctionContextScope(old)
    }

    pub fn enter_block_scope(&mut self, allow_param_shadow: bool) -> BlockScopeGuard {
        self.param_shadow_stack.push(allow_param_shadow);
        BlockScopeGuard
    }

    pub const fn enter_await_restriction(&mut self) -> AwaitRestrictionGuard {
        self.await_restriction_depth += 1;
        AwaitRestrictionGuard
    }

    pub const fn enter_super_property_scope(&mut self) -> SuperPropertyScopeGuard {
        self.super_property_scope += 1;
        SuperPropertyScopeGuard
    }

    pub const fn enter_super_call_scope(&mut self) -> SuperCallScopeGuard {
        self.super_call_scope += 1;
        SuperCallScopeGuard
    }

    pub const fn enter_relaxed_await_scope(&mut self) -> RelaxedAwaitGuard {
        self.await_relax_depth += 1;
        RelaxedAwaitGuard
    }

    const fn current_function_context(&self) -> Option<&FunctionContext> {
        self.function_ctx.as_ref()
    }

    #[must_use]
    pub fn is_await_restricted(&self) -> bool {
        (self.await_restriction_depth > 0 && self.await_relax_depth == 0)
            || self
                .current_function_context()
                .is_some_and(|ctx| ctx.await_restricted)
    }

    #[must_use]
    pub fn is_yield_restricted(&self) -> bool {
        self.current_function_context()
            .is_some_and(|ctx| ctx.is_generator)
    }

    #[must_use]
    pub fn in_async_function(&self) -> bool {
        self.current_function_context()
            .is_some_and(|ctx| ctx.is_async)
    }

    #[must_use]
    pub fn in_generator_function(&self) -> bool {
        self.current_function_context()
            .is_some_and(|ctx| ctx.is_generator)
    }

    #[must_use]
    pub const fn in_function_context(&self) -> bool {
        self.function_ctx.is_some()
    }

    #[must_use]
    pub fn is_private_name_known(&self, name: &str) -> bool {
        self.private_names
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }

    pub fn register_param_name(&mut self, name: &str) {
        if let Some(ctx) = self.function_ctx.as_mut() {
            ctx.param_names.insert(name.to_string());
        }
    }

    #[must_use] pub fn is_function_param_name(&self, name: &str) -> bool {
        self.function_ctx
            .as_ref()
            .is_some_and(|ctx| ctx.param_names.contains(name))
    }

    pub fn ensure_not_function_param(&self, name: &str) -> Result<(), String> {
        if self.param_shadow_stack.last().copied().unwrap_or(true) {
            return Ok(());
        }

        if self.is_function_param_name(name) {
            Err(format!("Identifier '{name}' has already been declared"))
        } else {
            Ok(())
        }
    }

    pub const fn set_current_function_strict(&mut self) {
        if let Some(ctx) = self.function_ctx.as_mut() {
            ctx.is_strict = true;
        }
    }

    #[must_use]
    pub fn in_strict_mode(&self) -> bool {
        self.function_ctx
            .as_ref()
            .map_or(self.script_strict, |ctx| ctx.is_strict)
    }

    #[must_use]
    pub fn can_use_super_property(&self) -> bool {
        self.super_property_scope > 0
            || self
                .function_ctx
                .as_ref()
                .is_some_and(|ctx| ctx.allow_super_property)
    }

    #[must_use]
    pub fn can_use_super_call(&self) -> bool {
        self.super_call_scope > 0
            || self
                .function_ctx
                .as_ref()
                .is_some_and(|ctx| ctx.allow_super_call)
    }

    pub const fn set_super_property_allowed(&mut self, allowed: bool) {
        if let Some(ctx) = self.function_ctx.as_mut() {
            ctx.allow_super_property = allowed;
        }
    }

    pub const fn set_super_call_allowed(&mut self, allowed: bool) {
        if let Some(ctx) = self.function_ctx.as_mut() {
            ctx.allow_super_call = allowed;
        }
    }
}

pub fn block_has_use_strict(block: &BlockStmt) -> bool {
    for stmt in &block.stmts {
        match stmt {
            Stmt::Empty(_) => continue,
            Stmt::Expr(ExprStmt { expr, .. }) => match &**expr {
                Expr::Lit(Lit::Str(str_lit)) if str_lit.value == *"use strict" => {
                    return true;
                }
                Expr::Lit(Lit::Str(_)) => continue,
                _ => break,
            },
            _ => break,
        }
    }

    false
}

pub fn statements_have_use_strict(stmts: &[Stmt]) -> bool {
    for stmt in stmts {
        match stmt {
            Stmt::Empty(_) => continue,
            Stmt::Expr(ExprStmt { expr, .. }) => match &**expr {
                Expr::Lit(Lit::Str(str_lit)) if str_lit.value == *"use strict" => {
                    return true;
                }
                Expr::Lit(Lit::Str(_)) => continue,
                _ => break,
            },
            _ => break,
        }
    }

    false
}

pub fn ensure_valid_identifier(name: &str) -> Result<(), String> {
    let mut chars = name.chars();

    let Some(first) = chars.next() else {
        return Err("Identifier cannot be empty".to_string());
    };

    if !is_valid_identifier_start(first) {
        return Err(format!(
            "Invalid identifier start character: U+{:04X}",
            first as u32
        ));
    }

    for ch in chars {
        if !is_valid_identifier_part(ch) {
            return Err(format!(
                "Invalid identifier part character: U+{:04X}",
                ch as u32
            ));
        }
    }

    Ok(())
}

fn is_valid_identifier_start(ch: char) -> bool {
    ch == '$' || ch == '_' || is_xid_start(ch) || is_other_id_start(ch)
}

fn is_valid_identifier_part(ch: char) -> bool {
    ch == '$'
        || is_xid_continue(ch)
        || matches!(ch, '\u{200C}' | '\u{200D}')
        || is_other_id_continue(ch)
        || is_other_id_start(ch)
}

pub fn single_stmt_contains_decl(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Decl(_) => true,
        Stmt::Labeled(labeled) => single_stmt_contains_decl(&labeled.body),
        _ => false,
    }
}

pub fn check_async_generator_fn_decl(stmt: &Stmt, context: &str) -> Result<(), String> {
    use swc_ecma_ast::Decl;

    if let Stmt::Decl(Decl::Fn(fn_decl)) = stmt {
        if fn_decl.function.is_async {
            return Err(format!(
                "Async function declaration is not allowed as the body of {context}"
            ));
        }
        if fn_decl.function.is_generator {
            return Err(format!(
                "Generator function declaration is not allowed as the body of {context}"
            ));
        }
    }
    Ok(())
}

pub fn is_labelled_function(stmt: &Stmt) -> bool {
    use swc_ecma_ast::Decl;

    match stmt {
        Stmt::Labeled(labeled) => match &*labeled.body {
            Stmt::Decl(Decl::Fn(_)) => true,
            other => is_labelled_function(other),
        },
        _ => false,
    }
}

pub fn is_reserved_word(value: &str) -> bool {
    matches!(
        value,
        "break"
            | "do"
            | "in"
            | "typeof"
            | "case"
            | "else"
            | "instanceof"
            | "var"
            | "catch"
            | "export"
            | "new"
            | "void"
            | "class"
            | "extends"
            | "return"
            | "while"
            | "const"
            | "finally"
            | "super"
            | "with"
            | "continue"
            | "for"
            | "switch"
            | "debugger"
            | "function"
            | "this"
            | "default"
            | "if"
            | "throw"
            | "delete"
            | "import"
            | "try"
            | "enum"
            | "false"
            | "true"
            | "null"
    )
}

/// Returns `true` for code points listed in ECMA-262 `Other_ID_Start`.
const fn is_other_id_start(ch: char) -> bool {
    matches!(
        ch,
        '\u{1885}' | '\u{1886}' | '\u{2118}' | '\u{212E}' | '\u{309B}' | '\u{309C}'
    )
}

/// Returns `true` for code points listed in ECMA-262 `Other_ID_Continue` (excluding
/// U+200C/U+200D which are handled separately for clarity).
fn is_other_id_continue(ch: char) -> bool {
    matches!(
        ch,
        '\u{00B7}' | '\u{0387}' | '\u{19DA}' | '\u{2054}' | '\u{FF3F}'
    ) || ('\u{1369}'..='\u{1371}').contains(&ch)
        || ('\u{203F}'..='\u{2040}').contains(&ch)
        || ('\u{FE33}'..='\u{FE34}').contains(&ch)
        || ('\u{FE4D}'..='\u{FE4F}').contains(&ch)
}
