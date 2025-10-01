use std::collections::HashSet;

use crate::Validator;
use swc_ecma_ast::Stmt;
use unicode_ident::{is_xid_continue, is_xid_start};

#[must_use]
pub struct PrivateNameScope;

#[derive(Clone, Copy, Default)]
pub struct FunctionContext {
    is_async: bool,
    is_generator: bool,
    await_restricted: bool,
}

#[must_use]
pub struct FunctionContextScope(Option<FunctionContext>);

impl PrivateNameScope {
    #[allow(clippy::unused_self)]
    pub fn exit(self, validator: &mut Validator<'_>) {
        validator.private_names.pop();
    }
}

impl FunctionContextScope {
    pub const fn exit(mut self, validator: &mut Validator) {
        validator.function_ctx = self.0.take();
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
        let previous = self.function_ctx;
        let await_restricted = is_async || previous.map_or(false, |ctx| ctx.await_restricted);

        let old = self.function_ctx.replace(FunctionContext {
            is_async,
            is_generator,
            await_restricted,
        });

        FunctionContextScope(old)
    }

    fn current_function_context(&self) -> FunctionContext {
        self.function_ctx.unwrap_or_default()
    }

    #[must_use]
    pub fn is_await_restricted(&self) -> bool {
        self.current_function_context().await_restricted
    }

    #[must_use]
    pub fn is_yield_restricted(&self) -> bool {
        self.current_function_context().is_generator
    }

    #[must_use]
    pub fn in_async_function(&self) -> bool {
        self.current_function_context().is_async
    }

    #[must_use]
    pub fn in_generator_function(&self) -> bool {
        self.current_function_context().is_generator
    }

    #[must_use]
    pub fn in_function_context(&self) -> bool {
        self.function_ctx.is_some()
    }

    #[must_use]
    pub fn is_private_name_known(&self, name: &str) -> bool {
        self.private_names
            .iter()
            .rev()
            .any(|scope| scope.contains(name))
    }
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
}

pub fn single_stmt_contains_decl(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Decl(_) => true,
        Stmt::Labeled(labeled) => single_stmt_contains_decl(&labeled.body),
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
fn is_other_id_start(ch: char) -> bool {
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
