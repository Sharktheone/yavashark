use anyhow::anyhow;
use swc_ecma_ast::Lit;
use yavashark_bytecode::ConstValue;
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;
use crate::{Compiler, Res};

impl Compiler {
    pub fn compile_lit(&mut self, lit: &Lit, out: Option<impl OutputData>) -> Res {
        if let Some(out) = out {
            let val = lit_to_const_value(lit)?;
            
            let c_idx = self.alloc_const(val);
            
            self.instructions.push(Instruction::move_(c_idx, out));
        }
        
        Ok(())
    }
}

pub fn lit_to_const_value(lit: &Lit) -> Res<ConstValue> {
    Ok(match lit {
        Lit::Str(s) => ConstValue::String(s.value.to_string()),
        Lit::Num(n) => ConstValue::Number(n.value),
        Lit::Bool(b) => ConstValue::Boolean(b.value),
        Lit::Null(_) => ConstValue::Null,
        Lit::BigInt(b) => ConstValue::BigInt((*b.value).clone()),
        Lit::Regex(r) => ConstValue::Regex(r.exp.to_string(), r.flags.to_string()),
        _ => return Err(anyhow!("Unsupported literal")),
    })
}