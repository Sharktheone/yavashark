use swc_ecma_ast::Stmt;

pub fn single_stmt_contains_decl(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Decl(_) => true,
        Stmt::Labeled(labeled) => single_stmt_contains_decl(&labeled.body),
        _ => false,
    }
}