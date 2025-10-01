use crate::Validator;
use crate::utils::{ensure_valid_identifier, is_reserved_word};
use swc_ecma_ast::{Ident, IdentName};

impl<'a> Validator<'a> {
    pub fn validate_ident(&mut self, ident: &Ident) -> Result<(), String> {
        let sym = ident.sym.as_str();

        ensure_valid_identifier(sym)?;

        if is_reserved_word(sym) {
            return Err(format!("Identifier '{}' is a reserved word", sym));
        }

        if sym == "await" && self.is_await_restricted() {
            return Err("Identifier 'await' is reserved in async functions".to_string());
        }

        if sym == "yield" && self.is_yield_restricted() {
            return Err("Identifier 'yield' is reserved in generator functions".to_string());
        }

        Ok(())
    }

    pub fn validate_ident_name(&mut self, ident: &IdentName) -> Result<(), String> {
        ensure_valid_identifier(ident.sym.as_str())
    }
}
