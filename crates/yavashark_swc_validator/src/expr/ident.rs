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
            return Err("Identifier 'await' is reserved in async contexts".to_string());
        }

        if sym == "yield" && self.is_yield_restricted() {
            return Err("Identifier 'yield' is reserved in generator functions".to_string());
        }

        if self.in_strict_mode()
            && matches!(
                sym,
                "implements"
                    | "interface"
                    | "let"
                    | "package"
                    | "private"
                    | "protected"
                    | "public"
                    | "static"
            )
        {
            return Err(format!("Identifier '{}' is reserved in strict mode", sym));
        }

        if self.in_strict_mode() && sym == "yield" {
            return Err("Identifier 'yield' is reserved in strict mode".to_string());
        }

        Ok(())
    }

    pub fn validate_ident_name(&mut self, ident: &IdentName) -> Result<(), String> {
        ensure_valid_identifier(ident.sym.as_str())
    }
}
