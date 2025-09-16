use swc_ecma_ast::DebuggerStmt;
use crate::Validator;

impl Validator {
    pub fn validate_debugger(_debugger: &DebuggerStmt) -> Result<(), String> {
        Ok(())
    }
}
