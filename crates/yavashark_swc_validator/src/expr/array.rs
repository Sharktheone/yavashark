use crate::Validator;
use swc_ecma_ast::ArrayLit;

impl<'a> Validator<'a> {
    pub fn validate_array_expr(&mut self, array: &'a ArrayLit) -> Result<(), String> {
        for elem in array.elems.iter().flatten() {
            self.validate_expr(&elem.expr)?;
        }

        Ok(())
    }
}
