use crate::Validator;
use swc_ecma_ast::PrivateName;

impl<'a> Validator<'a> {
    pub fn validate_private_name_expr(&mut self, private_name: &PrivateName) -> Result<(), String> {
        if private_name.name.as_str().starts_with('\u{200D}') {
            return Err(format!(
                "Identifier cannot start with a zero-width joiner (U+200D): {}",
                private_name.name
            ));
        }

        if !self.is_private_name_known(private_name.name.as_str()) {
            return Err(format!("Unknown private name: #{}", private_name.name));
        }

        Ok(())
    }
}
