use crate::Validator;
use swc_ecma_ast::Tpl;

impl<'a> Validator<'a> {
    pub fn validate_tpl_expr(&mut self, tpl: &'a Tpl) -> Result<(), String> {
        for quasi in &tpl.quasis {
            let raw = quasi.raw.as_ref();

            if raw.contains("\\8") || raw.contains("\\9") {
                return Err(
                    "Invalid escape sequence in template literal: \\8 and \\9 are not allowed"
                        .to_string(),
                );
            }

            if raw.contains("\\u{") && raw.contains('_')
                && let Some(start) = raw.find("\\u{")
                    && let Some(end) = raw[start..].find('}') {
                        let unicode_part = &raw[start..=(start + end)];
                        if unicode_part.contains('_') {
                            return Err("Invalid escape sequence in template literal: numeric separators are not allowed in unicode escape sequences".to_string());
                        }
                    }

            if quasi.cooked.is_none() {
                return Err("Invalid escape sequence in template literal".to_string());
            }
        }

        for expr in &tpl.exprs {
            self.validate_expr(expr)?;
        }
        Ok(())
    }
}
