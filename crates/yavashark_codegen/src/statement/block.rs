use swc_ecma_ast::BlockStmt;

use crate::{ByteCodegen, Res};

impl ByteCodegen {
    pub fn compile_block(&mut self, stmt: &BlockStmt) -> Res {
        for stmt in &stmt.stmts {
            self.compile_statement(stmt)?;
        }

        Ok(())
    }
}
