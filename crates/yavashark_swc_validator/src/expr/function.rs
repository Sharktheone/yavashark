use crate::Validator;
use swc_ecma_ast::{FnExpr, Function};

impl Validator {
    pub fn validate_function_expr(function: &FnExpr) -> Result<(), String> {
        Self::validate_function(&function.function)
    }

    pub fn validate_function(function: &Function) -> Result<(), String> {
        for param in &function.params {
            Self::validate_pat(&param.pat)?;
        }

        if let Some(body) = &function.body {
            Self::validate_block(body)?;
        }

        Ok(())
    }
}
