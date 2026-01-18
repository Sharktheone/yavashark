use crate::{ByteCodegen, Res};
use anyhow::anyhow;
use swc_ecma_ast::Lit;
use yavashark_bytecode::ConstValue;

impl ByteCodegen {
    pub fn compile_lit(&mut self, stmt: &Lit) -> Res {
        let val = match stmt {
            Lit::Str(s) => ConstValue::String(s.value.to_string_lossy().into_owned()),
            Lit::Num(n) => ConstValue::Number(n.value),
            Lit::Bool(b) => ConstValue::Boolean(b.value),
            Lit::Null(_) => ConstValue::Null,
            Lit::BigInt(_) => todo!("Big int lit"),
            Lit::Regex(_) => todo!("Regex lit"),
            Lit::JSXText(_) => Err(anyhow!("JSXText is not supported"))?,
        };

        let lit = self.allocate_literal(val);

        self.instructions
            .push(yavashark_bytecode::Instruction::LdaAcc(lit));

        Ok(())
    }
}
