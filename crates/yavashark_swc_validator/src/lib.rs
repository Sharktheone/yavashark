use swc_ecma_ast::{ModuleItem, Stmt};

pub fn validate_statements(ast: &[Stmt]) -> Result<(), String> {
    for stmt in ast {
        validate_statement(stmt)?;
    }
    Ok(())
}

pub fn validate_statement(stmt: &Stmt) -> Result<(), String> {
    Ok(())
}


pub fn validate_module_items(ast: &[ModuleItem]) -> Result<(), String> {
    for item in ast {
        match item {
            ModuleItem::Stmt(stmt) => validate_statement(stmt)?,
            ModuleItem::ModuleDecl(item) => validate_module_decl(item)?,
        }
    }
    Ok(())
}

pub fn validate_module_decl(decl: &swc_ecma_ast::ModuleDecl) -> Result<(), String> {
    Ok(())
}