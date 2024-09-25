use crate::{ByteCodegen, Res};
use swc_ecma_ast::{MemberExpr, MemberProp};
use yavashark_bytecode::Instruction;

impl ByteCodegen {
    pub fn compile_member(&mut self, stmt: &MemberExpr) -> Res {
        self.compile_expr(&stmt.obj, stmt.span)?;

        if let MemberProp::Ident(ident) = &stmt.prop {
            let var_name = self.allocate_variable(ident.sym.as_str().to_string());

            self.instructions.push(Instruction::LoadMemberAcc(var_name));
        } else {
            todo!()
        }

        Ok(())
    }
}
