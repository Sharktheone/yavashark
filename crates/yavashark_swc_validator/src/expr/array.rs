use swc_ecma_ast::ArrayLit;
use crate::Validator;

impl Validator {
    pub fn validate_array_expr(array: &ArrayLit) -> Result<(), String> {
        for elem in array.elems.iter().flatten() {
            Self::validate_expr(&elem.expr)?;
        }

        Ok(())
    }
}
