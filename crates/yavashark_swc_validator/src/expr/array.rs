use crate::Validator;
use swc_ecma_ast::ArrayLit;

impl Validator {
    pub fn validate_array_expr(array: &ArrayLit) -> Result<(), String> {
        for elem in array.elems.iter().flatten() {
            Self::validate_expr(&elem.expr)?;
        }

        Ok(())
    }
}
