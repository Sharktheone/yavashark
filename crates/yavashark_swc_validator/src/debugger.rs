use crate::Validator;
use swc_ecma_ast::DebuggerStmt;

impl<'a> Validator<'a> {
    pub fn validate_debugger(&mut self, _debugger: &DebuggerStmt) -> Result<(), String> {
        Ok(())
    }
}
