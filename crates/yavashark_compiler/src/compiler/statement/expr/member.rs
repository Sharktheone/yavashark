use crate::{Compiler, Res};
use swc_ecma_ast::{MemberExpr, MemberProp};
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::data::{DataType, OutputData};
use yavashark_bytecode::instructions::Instruction;


#[derive(Debug, Clone, Copy)]
pub enum MemberKey {
    Public(DataType),
    Private(DataType),
}

impl MemberKey {
    pub fn data_type(&self) -> DataType {
        match self {
            MemberKey::Public(dt) | MemberKey::Private(dt) => *dt,
        }
    }
}

impl Compiler {
    pub fn compile_member_prop(&mut self, m: &MemberProp) -> Res<MemberKey> {
        Ok(match m {
            MemberProp::Ident(ident) => {
                MemberKey::Public(self.alloc_const(ConstValue::String(ident.sym.as_str().to_string()))
                    .into()) //TODO: this should rather be stored in var names
            }
            MemberProp::Computed(expr) => {
                let out = self.alloc_reg_or_stack();
                MemberKey::Public(self.compile_expr_data(&expr.expr, Some(out))?)
            }
            MemberProp::PrivateName(name) => {
                MemberKey::Private(self.alloc_const(ConstValue::String(name.name.to_string()))
                    .into()) //TODO: this should rather be stored in var names
            }
        })
    }

    pub fn compile_member(&mut self, expr: &MemberExpr, out: Option<impl OutputData>) -> Res {
        let Some(out) = out else { return Ok(()) };

        let member = self.compile_member_prop(&expr.prop)?;
        let prop = self.compile_expr_data_acc(&expr.obj)?;


        match member {
            MemberKey::Public(member) => {
                self.instructions
                    .push(Instruction::load_member(prop, member, out));
            }
            MemberKey::Private(member) => {
                self.instructions
                    .push(Instruction::load_private_member(prop, member, out));
            }
        }

        self.dealloc(prop);
        self.dealloc(member.data_type());

        Ok(())
    }
}
