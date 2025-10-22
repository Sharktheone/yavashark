use std::collections::HashSet;
use crate::Validator;
use swc_ecma_ast::SwitchStmt;

impl<'a> Validator<'a> {
    pub fn validate_switch(&mut self, brk: &'a SwitchStmt) -> Result<(), String> {

        let mut lexical = Vec::new();
        let mut var_names = Vec::new();

        self.validate_expr(&brk.discriminant)?;
        for case in &brk.cases {
            if let Some(test) = &case.test {
                self.validate_expr(test)?;
            }
            self.validate_statements(&case.cons)?;


            for stmt in &case.cons{
                crate::block::collect_lexical_names(stmt, &mut lexical);
                crate::block::collect_var_declared_names(stmt, &mut var_names);
            }
        }

        let mut seen = HashSet::new();
        for name in &lexical {
            if *name == "_" {
                continue;
            }
            if !seen.insert(*name) {
                return Err(format!(
                    "Identifier '{name}' has already been declared in this block"
                ));
            }
        }

        if !lexical.is_empty() && !var_names.is_empty() {
            let set_lex: HashSet<&str> = lexical
                .iter()
                .copied()
                .filter(|name| *name != "_")
                .collect();
            if let Some(name) = var_names
                .iter()
                .copied()
                .filter(|name| *name != "_")
                .find(|name| set_lex.contains(name))
            {
                return Err(format!(
                    "Identifier '{name}' conflicts with lexical declaration in this block"
                ));
            }
        }

        Ok(())
    }
}
