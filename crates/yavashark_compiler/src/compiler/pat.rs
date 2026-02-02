use crate::{Compiler, Res};
use anyhow::anyhow;
use std::mem;
use std::path::Component;
use std::rc::Rc;
use swc_ecma_ast::{ArrayPat, AssignPat, Expr, ObjectPat, ObjectPatProp, Pat, PropName};
use yavashark_bytecode::data::{Acc, Data, DataType, F32, OutputDataType, Undefined, VarName};
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::{ConstValue, DataTypeValue};

impl Compiler {
    pub fn compile_pat_var(&mut self, pat: &Pat, source: impl Data) -> Res {
        self.compile_pat(pat, source, &mut |compiler, dtype, name| {
            compiler
                .instructions
                .push(Instruction::decl_var(dtype, name));
        })
    }

    pub fn compile_pat_let(&mut self, pat: &Pat, source: impl Data) -> Res {
        self.compile_pat(pat, source, &mut |compiler, dtype, name| {
            compiler
                .instructions
                .push(Instruction::decl_let(dtype, name));
        })
    }

    pub fn compile_pat_const(&mut self, pat: &Pat, source: impl Data) -> Res {
        self.compile_pat(pat, source, &mut |compiler, dtype, name| {
            compiler
                .instructions
                .push(Instruction::decl_const(dtype, name));
        })
    }

    pub fn compile_pat(
        &mut self,
        pat: &Pat,
        source: impl Data,
        cb: &mut impl FnMut(&mut Self, DataType, VarName),
    ) -> Res {
        match pat {
            Pat::Array(array) => self.compile_array_pat(array, source, cb)?,
            Pat::Ident(ident) => {
                let name = self.alloc_var(ident.as_ref());

                cb(self, source.data_type(), name);
            }
            Pat::Assign(assign) => self.compile_assign_pat(assign, source, cb)?,
            Pat::Object(obj) => self.compile_object_pat(obj, source, cb)?,
            Pat::Invalid(invalid) => Err(anyhow!("Invalid pattern: {:?}", invalid))?,
            Pat::Expr(expr) => self.compile_expr_pat(expr, source)?,
            Pat::Rest(_) => todo!(),
        }

        Ok(())
    }

    pub fn compile_array_pat(
        &mut self,
        array: &ArrayPat,
        source: impl Data,
        cb: &mut impl FnMut(&mut Self, DataType, VarName),
    ) -> Res {
        let iter = self.alloc_reg_or_stack();

        self.instructions.push(Instruction::push_iter(source, iter));

        let out = self.alloc_reg_or_stack();
        let mut close = true;

        let mut patch_insn = Vec::new();

        let dbg_from = self.instructions.len();

        for (i, elem) in array.elems.iter().enumerate() {
            if let Some(elem) = elem {
                if let Pat::Rest(rest) = elem {
                    let rest_out = self.alloc_reg_or_stack();
                    self.instructions
                        .push(Instruction::iter_collect(iter, rest_out));

                    close = false;

                    self.compile_pat(&rest.arg, rest_out, cb)?;
                } else {
                    patch_insn.push((self.instructions.len(), true, Some(elem)));
                    self.instructions.push(Instruction::iter_next(iter, out));
                    self.compile_pat(elem, out, cb)?;
                }
            } else {
                patch_insn.push((self.instructions.len(), false, None));
                self.instructions
                    .push(Instruction::iter_next_no_output(iter));
            }
        }

        if close || !patch_insn.is_empty() {
            self.instructions.push(Instruction::iter_close(iter));
        }

        let idx = self.instructions.len();
        self.instructions.push(Instruction::jmp(0));
        let mut emitted = false;

        for (idx, capture_output, pat) in patch_insn {
            let jmp_target = self.instructions.len();
            self.instructions[idx] = if capture_output {
                Instruction::iter_next_jmp(iter, jmp_target, out)
            } else {
                Instruction::iter_next_no_output_jmp(iter, jmp_target)
            };

            if let Some(pat) = pat {
                emitted = true;
                self.compile_pat(pat, Undefined, cb)?;
            }
        }

        if emitted {
            self.instructions[idx] = Instruction::jmp(self.instructions.len());
        } else {
            self.instructions.pop();
        }

        self.dealloc(iter);
        self.dealloc(out);

        Ok(())
    }

    pub fn compile_object_pat(
        &mut self,
        obj: &ObjectPat,
        source: impl Data,
        cb: &mut impl FnMut(&mut Self, DataType, VarName),
    ) -> Res {
        let has_rest = obj
            .props
            .last()
            .is_some_and(|p| matches!(p, ObjectPatProp::Rest(_)));

        if has_rest {
            self.instructions
                .push(Instruction::begin_spread(obj.props.len()));
        }

        for (i, prop) in obj.props.iter().enumerate() {
            match prop {
                ObjectPatProp::KeyValue(prop) => {
                    let mut dealloc = Vec::new();
                    let key = self.convert_pat_prop_name(&prop.key, &mut dealloc)?;

                    if has_rest {
                        self.instructions.push(Instruction::push_spread(key));
                    }
                    self.instructions
                        .push(Instruction::load_member(source, key, Acc));

                    self.compile_pat(&prop.value, Acc, cb)?;

                    for d in dealloc {
                        self.dealloc(d);
                    }
                }
                ObjectPatProp::Assign(prop) => {
                    if let Some(value) = &prop.value {
                        let name = self.alloc_var(prop.key.id.sym.as_ref());
                        let key = self.alloc_const(prop.key.sym.as_str());

                        if has_rest {
                            self.instructions.push(Instruction::push_spread(key));
                        }

                        self.instructions
                            .push(Instruction::load_member(source, key, Acc));

                        let idx = self.instructions.len();
                        self.instructions.push(Instruction::jmp(0));

                        let mut name_restore = None;
                        if Self::expr_should_be_named(&value) {
                            name_restore = mem::replace(
                                &mut self.current_fn_name,
                                Some(prop.key.id.sym.to_string()),
                            );
                        }

                        self.compile_expr_data_certain(value, Acc);
                        self.current_fn_name = name_restore;

                        self.instructions[idx] =
                            Instruction::jmp_if_not_undefined(Acc, self.instructions.len());

                        cb(self, Acc.into(), name);
                    } else {
                        let name = self.alloc_var(prop.key.id.as_ref());
                        let key = self.alloc_const(prop.key.sym.as_str());

                        if has_rest {
                            self.instructions.push(Instruction::push_spread(key));
                        }

                        self.instructions
                            .push(Instruction::load_member(source, key, Acc));

                        cb(self, Acc.into(), name);
                    }
                }
                ObjectPatProp::Rest(prop) => {
                    if i != obj.props.len() - 1 {
                        return Err(anyhow!(
                            "Rest element must be the last element in an object pattern"
                        ));
                    }

                    let rest_out = self.alloc_reg_or_stack();

                    self.instructions
                        .push(Instruction::end_spread(source, rest_out));

                    self.compile_pat(&prop.arg, rest_out, cb)?;
                }
            }
        }

        if obj.props.is_empty() {
            self.instructions
                .push(Instruction::throw_if_not_object(source));
        }

        Ok(())
    }

    pub fn compile_assign_pat(
        &mut self,
        assign: &AssignPat,
        source: impl Data,
        cb: &mut impl FnMut(&mut Self, DataType, VarName),
    ) -> Res {
        let out = self.alloc_reg_or_stack();
        self.instructions.push(Instruction::move_(source, out));

        let idx = self.instructions.len();
        self.instructions.push(Instruction::jmp(0));

        let mut name_restore = None;

        if let Pat::Ident(fn_name) = &*assign.left {
            if Self::expr_should_be_named(&assign.right) {
                let name = fn_name.sym.to_string();
                name_restore = mem::replace(&mut self.current_fn_name, Some(name));
            }
        }

        self.compile_expr_data_certain(&assign.right, out)?;

        self.current_fn_name = name_restore;

        self.instructions[idx] = Instruction::jmp_if_not_undefined(out, self.instructions.len());

        self.compile_pat(&assign.left, out, cb)?;
        self.dealloc(out);

        Ok(())
    }

    pub fn compile_expr_pat(&mut self, expr: &Expr, source: impl Data) -> Res {
        self.compile_assign_expr(expr, source)?;

        Ok(())
    }

    pub fn compile_define_pat_variables(
        &mut self,
        pat: &Pat,
        cb: &mut impl FnMut(&mut Self, DataType, VarName),
    ) -> Res {
        match pat {
            Pat::Ident(id) => {
                let var_name = self.alloc_var(id.sym.as_str());

                cb(self, DataType::Undefined(Undefined), var_name);
            }
            Pat::Array(array) => {
                for elem in &array.elems {
                    if let Some(pat) = elem {
                        self.compile_define_pat_variables(pat, cb)?;
                    }
                }
            }
            Pat::Rest(rest) => {
                //TODO: this should probably just be an empty array
                self.compile_define_pat_variables(&rest.arg, cb)?;
            }
            Pat::Object(object) => {
                for prop in &object.props {
                    match prop {
                        ObjectPatProp::KeyValue(kv) => {
                            self.compile_define_pat_variables(&kv.value, cb)?;
                        }
                        ObjectPatProp::Assign(assign) => {
                            let var_name = self.alloc_var(assign.key.sym.as_str());

                            let dt = if let Some(expr) = &assign.value {
                                let mut name_restore = None;
                                if Self::expr_should_be_named(&expr) {
                                    name_restore = mem::replace(
                                        &mut self.current_fn_name,
                                        Some(assign.key.id.sym.to_string()),
                                    );
                                }

                                self.compile_expr_data_certain(expr, Acc);
                                self.current_fn_name = name_restore;

                                DataType::Acc(Acc)
                            } else {
                                DataType::Undefined(Undefined)
                            };

                            cb(self, dt, var_name);
                        }
                        ObjectPatProp::Rest(rest) => {
                            self.compile_define_pat_variables(&rest.arg, cb)?;
                        }
                    }
                }
            }
            Pat::Assign(assign) => {
                let mut name_restore = None;

                if let Pat::Ident(fn_name) = &*assign.left {
                    if Self::expr_should_be_named(&assign.right) {
                        let name = fn_name.sym.to_string();
                        name_restore = mem::replace(&mut self.current_fn_name, Some(name));
                    }
                }

                self.compile_expr_data_certain(&assign.right, Acc)?;

                self.current_fn_name = name_restore;

                self.compile_pat(&assign.left, Acc, cb)?;
            }

            Pat::Invalid(invalid) => Err(anyhow!("Invalid pattern: {:?}", invalid))?,
            Pat::Expr(expr) => self.compile_expr_pat(expr, Undefined)?,
        }

        Ok(())
    }

    pub fn convert_pat_prop_name(
        &mut self,
        key: &PropName,
        dealloc: &mut Vec<OutputDataType>,
    ) -> anyhow::Result<DataType> {
        Ok(match key {
            PropName::Ident(id) => {
                let id = id.sym.to_string();

                let c = self.alloc_const(id);

                DataType::Const(c)
            }
            PropName::Str(s) => {
                let Some(s) = s.value.as_str() else {
                    return Err(anyhow!("Invalid string in property name"));
                };

                let c = self.alloc_const(s);

                DataType::Const(c)
            }
            PropName::Num(n) => DataType::F32(F32(n.value as f32)),
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
        })
    }

    pub fn expr_should_be_named(expr: &Expr) -> bool {
        match expr {
            Expr::Fn(_) => true,
            Expr::Arrow(_) => true,
            Expr::Paren(paren) => Self::expr_should_be_named(&paren.expr),
            _ => false,
        }
    }
}
