use swc_ecma_ast::Stmt;

pub fn validate_statements(ast: &[Stmt]) -> Result<(), String> {
    for stmt in ast {
        validate_statement(stmt)?;
    }
    Ok(())
}

pub fn validate_statement(stmt: &Stmt) -> Result<(), String> {
    Ok(())
}