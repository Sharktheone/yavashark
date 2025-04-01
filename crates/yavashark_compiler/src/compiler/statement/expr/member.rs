use swc_ecma_ast::{MemberExpr, MemberProp};
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_member(&mut self, expr: &MemberExpr, out: Option<impl OutputData>) -> Res {
        let Some(out) = out else {
            return Ok(())
        };
        
        let member = match &expr.prop {
            MemberProp::Ident(ident) => {
                self.alloc_const(ConstValue::String(ident.sym.as_str().to_string())).into() //TODO: this should rather be stored in var names
            }
            MemberProp::Computed(expr) => {
                let out = self.alloc_reg_or_stack();
                self.compile_expr_data(&expr.expr, Some(out))?
            }
            MemberProp::PrivateName(_) => {
                todo!()
            }
        };
        
        
        let prop = self.compile_expr_data_acc(&expr.obj)?;
        
        self.instructions.push(Instruction::load_member(prop, member, out));
        self.dealloc(prop);
        self.dealloc(member);
        
        Ok(())
    }
}