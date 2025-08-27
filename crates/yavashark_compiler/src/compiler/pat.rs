use crate::{Compiler, Res};
use anyhow::anyhow;
use std::path::Component;
use std::rc::Rc;
use swc_ecma_ast::{ArrayPat, AssignPat, ObjectPat, ObjectPatProp, Pat, PropName};
use yavashark_bytecode::data::{Acc, Data, DataType, OutputDataType, VarName, F32};
use yavashark_bytecode::{ConstValue, DataTypeValue};
use yavashark_bytecode::instructions::Instruction;



impl Compiler {
    pub fn compile_pat_var(&mut self, pat: &Pat, source: impl Data) -> Res {
        self.compile_pat(pat, source, &mut |compiler, dtype, name| {
            compiler.instructions.push(Instruction::decl_var(dtype, name));
        })
    }

    pub fn compile_pat_let(&mut self, pat: &Pat, source: impl Data) -> Res {
        self.compile_pat(pat, source, &mut |compiler, dtype, name| {
            compiler.instructions.push(Instruction::decl_let(dtype, name));
        })
    }

    pub fn compile_pat_const(&mut self, pat: &Pat, source: impl Data) -> Res {
        self.compile_pat(pat, source, &mut |compiler, dtype, name| {
            compiler.instructions.push(Instruction::decl_const(dtype, name));
        })
    }

    pub fn compile_pat(&mut self, pat: &Pat, source: impl Data, cb: &mut impl FnMut(&mut Compiler, DataType, VarName)) -> Res {
        match pat {
            Pat::Array(array) => self.compile_array_pat(array, source, cb)?,
            Pat::Ident(ident) => {
                let name = self.alloc_var(ident.as_ref());

                cb(self, source.data_type(), name);
            },
            Pat::Assign(assign) => self.compile_assign_pat(assign, source, cb)?,
            Pat::Object(obj) => self.compile_object_pat(obj, source, cb)?,
            Pat::Invalid(invalid) => Err(anyhow!("Invalid pattern: {:?}", invalid))?,
            _ => todo!(),
        }

        Ok(())
    }

    pub fn compile_array_pat(&mut self, array: &ArrayPat, source: impl Data, cb: &mut impl FnMut(&mut Compiler, DataType, VarName)) -> Res {
        let iter = self.alloc_reg_or_stack();

        self.instructions.push(Instruction::push_iter(source, iter));

        let out = self.alloc_reg_or_stack();
        for (i, elem) in array.elems.iter().enumerate() {
            self.instructions.push(Instruction::iter_next(iter, out));

            if let Some(elem) = elem {
                self.compile_pat(elem, out, cb)?;
            } else {
                self.instructions
                    .push(Instruction::iter_next_no_output(iter))
            }
        }

        self.dealloc(iter);
        self.dealloc(out);

        Ok(())
    }

    pub fn compile_object_pat(&mut self, obj: &ObjectPat, source: impl Data, cb: &mut impl FnMut(&mut Compiler, DataType, VarName)) -> Res {
        let mut dealloc = Vec::new();

        for prop in &obj.props {
            match prop {
                ObjectPatProp::KeyValue(prop) => {
                    let key = self.convert_pat_prop_name(&prop.key, &mut dealloc);

                    self.instructions.push(Instruction::load_member(source, key, Acc));

                    self.compile_pat(&prop.value, Acc, cb)?;
                }
                ObjectPatProp::Assign(prop) => {
                    if let Some(value) = &prop.value {
                        let name = self.alloc_var(prop.key.id.as_ref());
                        let key = self.alloc_const(prop.key.sym.as_str());

                        self.instructions.push(Instruction::load_member(source, key, Acc));

                        let idx = self.instructions.len();
                        self.instructions.push(Instruction::jmp(0));

                        self.compile_expr_data_certain(value, Acc);

                        self.instructions[idx] = Instruction::jmp_if_not_undefined(Acc, self.instructions.len());

                        cb(self, Acc.into(), name);
                    } else {
                        let name = self.alloc_var(prop.key.id.as_ref());
                        let key = self.alloc_const(prop.key.sym.as_str());

                        self.instructions.push(Instruction::load_member(source, key, Acc));

                        cb(self, Acc.into(), name);
                    }
                }
                ObjectPatProp::Rest(prop) => todo!()
            }
        }

        if obj.props.is_empty() {
            //TODO: throw if not an object
        }


        Ok(())
    }

    pub fn compile_assign_pat(&mut self, assign: &AssignPat, source: impl Data, cb: &mut impl FnMut(&mut Compiler, DataType, VarName)) -> Res {
        let out = self.alloc_reg_or_stack();
        self.instructions.push(Instruction::move_(source, out));

        let idx = self.instructions.len();
        self.instructions.push(Instruction::jmp(0));

        self.compile_expr_data_certain(&assign.right, out)?;
        self.instructions[idx] = Instruction::jmp_if_not_undefined(out, self.instructions.len());

        self.compile_pat(&assign.left, out, cb)?;
        self.dealloc(out);

        Ok(())
    }


    pub fn convert_pat_prop_name(
        &mut self,
        key: &PropName,
        dealloc: &mut Vec<OutputDataType>,
    ) -> DataType {
        match key {
            PropName::Ident(id) => {
                let id = id.sym.to_string();

                let c = self.alloc_const(id);

                DataType::Const(c)
            },
            PropName::Str(s) => {
                let s = s.value.to_string();

                let c = self.alloc_const(s);

                DataType::Const(c)
            }
            PropName::Num(n) => {

                DataType::F32(F32(n.value as f32))
            }
            PropName::Computed(c) => {
                let reg = self.alloc_reg_or_stack();
                self.compile_expr_data_certain(&c.expr, reg);
                dealloc.push(reg);

                reg.into()
            }
            PropName::BigInt(b) => {
                let b = Rc::new((*b.value).clone());

                let c = self.alloc_const(ConstValue::BigInt(b));

                DataType::Const(c)
            }
        }
    }
}

