use crate::{Validator, utils::ensure_valid_identifier};
use swc_ecma_ast::PrivateName;

impl<'a> Validator<'a> {
    pub fn validate_private_name_expr(&mut self, private_name: &PrivateName) -> Result<(), String> {
        let name = private_name.name.as_str();

        if let Err(err) = ensure_valid_identifier(name) {
            return Err(format!("Invalid private identifier: {err}"));
        }

        if !self.is_private_name_known(name) {
            return Err(format!("Unknown private name: #{}", private_name.name));
        }

        Ok(())
    }
}
