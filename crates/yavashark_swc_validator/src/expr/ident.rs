use swc_ecma_ast::{Ident, IdentName};
use crate::Validator;

impl Validator {
    pub fn validate_ident(ident: &Ident) -> Result<(), String> {
        if ident.sym.as_str().starts_with('\u{200D}') {
            return Err(format!("Identifier cannot start with a zero-width joiner (U+200D): {}", ident.sym));
        }

        Ok(())
    }

    pub fn validate_ident_name(ident: &IdentName) -> Result<(), String> {
        if ident.sym.as_str().starts_with('\u{200D}') {
            return Err(format!("Identifier cannot start with a zero-width joiner (U+200D): {}", ident.sym));
        }

        Ok(())
    }
}
