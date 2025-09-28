use std::collections::HashSet;

use swc_ecma_ast::Stmt;
use unicode_ident::{is_xid_continue, is_xid_start};
use crate::Validator;

#[must_use]
pub struct PrivateNameScope;

#[derive(Clone, Copy, Default)]
pub struct FunctionContext {
    is_async: bool,
    is_generator: bool,
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

    pub const fn enter_function_context(&mut self, is_async: bool, is_generator: bool) -> FunctionContextScope {
        let old = self.function_ctx.replace(FunctionContext { is_async, is_generator });

        FunctionContextScope(old)
    }

    fn current_function_context(&self) -> FunctionContext {
        self.function_ctx.unwrap_or_default()
    }

    #[must_use]
    pub fn is_await_restricted(&self) -> bool {
        self.current_function_context().is_async
    }

    #[must_use]
    pub fn is_yield_restricted(&self) -> bool {
        self.current_function_context().is_generator
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
    ch == '$' || ch == '_' || is_xid_start(ch)
}

fn is_valid_identifier_part(ch: char) -> bool {
    ch == '$' || is_xid_continue(ch) || matches!(ch, '\u{200C}' | '\u{200D}')
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