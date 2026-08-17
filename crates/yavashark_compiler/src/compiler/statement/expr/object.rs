use super::MoveOptimization;
use crate::{Compiler, Res};
use anyhow::anyhow;
use std::rc::Rc;
use swc_ecma_ast::{ObjectLit, Param, Prop, PropName, PropOrSpread};
use yavashark_bytecode::data::{OutputData, OutputDataType};
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::{BytecodeFunctionCode, ConstValue, DataTypeValue, FunctionBlueprint, ObjectLiteralBlueprint};

impl Compiler {
    pub fn compile_object(
        &mut self,
        expr: &ObjectLit,
        out: Option<impl OutputData>,
    ) -> Res<Option<MoveOptimization>> {
        let Some(out) = out else {
            return Ok(None);
        };

        let mut properties = Vec::with_capacity(expr.props.len());
        let mut dealloc = Vec::new();

        for prop in &expr.props {
            match prop {
                PropOrSpread::Prop(p) => match &**p {
                    Prop::Shorthand(ident) => {
                        let var = self.alloc_var(ident.sym.as_str());
                        let dt = DataTypeValue::Var(var);
                        let id = DataTypeValue::String(ident.sym.to_string());
                        properties.push((id, dt));
                    }
                    Prop::KeyValue(kv) => {
                        let prop = self.convert_prop_name(&kv.key, &mut dealloc)?;

                        let storage = self.alloc_reg_or_stack();
                        dealloc.push(storage);

                        self.compile_expr_data_certain(&kv.value, storage);
                        properties.push((prop, storage.into()));
                    }
                    Prop::Getter(g) => {
                        let prop = self.convert_prop_name(&g.key, &mut dealloc)?;

                        let storage = self.alloc_reg_or_stack();
                        dealloc.push(storage);

                        let bp = self.create_function_blueprint(&g.function, None)?;

                        properties.push((prop, DataTypeValue::Get(bp)));
                    }
                    Prop::Setter(s) => {
                        let prop = self.convert_prop_name(&s.key, &mut dealloc)?;

                        let storage = self.alloc_reg_or_stack();
                        dealloc.push(storage);

                        let bp = self.create_function_blueprint(&s.function, None)?;

                        properties.push((prop, DataTypeValue::Set(bp)));
                    }
                    Prop::Method(m) => {
                        let prop = self.convert_prop_name(&m.key, &mut dealloc)?;

                        let storage = self.alloc_reg_or_stack();
                        dealloc.push(storage);

                        let bp = self.create_function_blueprint(&m.function, None)?;

                        properties.push((prop, DataTypeValue::Function(bp)));
                    }
                    Prop::Assign(_) => todo!(),
                },
                PropOrSpread::Spread(_) => {
                    todo!()
                }
            }
        }
        let ob = self.alloc_const(ConstValue::Object(ObjectLiteralBlueprint { properties }));

        let m = MoveOptimization::new(ob, vec![Instruction::move_(ob, out)]);

        for dealloc in dealloc {
            self.dealloc(dealloc);
        }

        Ok(Some(m))
    }

    pub fn convert_prop_name(
        &mut self,
        key: &PropName,
        dealloc: &mut Vec<OutputDataType>,
    ) -> Res<DataTypeValue> {
        Ok(match key {
            PropName::Ident(id) => DataTypeValue::String(id.sym.to_string()),
            PropName::Str(s) => DataTypeValue::String(s.value.to_string_lossy().into_owned()),
            PropName::Num(n) => DataTypeValue::Number(n.value),
            PropName::Computed(c) => {
                let reg = self.alloc_reg_or_stack();
                self.compile_expr_data_certain(&c.expr, reg);
                dealloc.push(reg);

                reg.into()
            }
            PropName::BigInt(b) => DataTypeValue::BigInt(Rc::new((*b.value).clone())),
        })
    }
}
