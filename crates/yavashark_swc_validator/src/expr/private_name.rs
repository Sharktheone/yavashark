use swc_ecma_ast::PrivateName;
use crate::Validator;

impl Validator {
    pub fn validate_private_name_expr(private_name: &PrivateName) -> Result<(), String> {
        if private_name.name.as_str().starts_with('\u{200D}') {
            return Err(format!("Identifier cannot start with a zero-width joiner (U+200D): {}", private_name.name));
        }

        Ok(())
    }
}
