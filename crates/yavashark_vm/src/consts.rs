use crate::function_code::{BytecodeArrowFunction, BytecodeFunction};
use crate::VM;
use std::cell::RefCell;
use yavashark_bytecode::{
    ArrayLiteralBlueprint, ConstValue, DataTypeValue, ObjectLiteralBlueprint,
};
use yavashark_env::array::Array;
use yavashark_env::builtins::RegExp;
use yavashark_env::optimizer::{FunctionCode, OptimFunction};
use yavashark_env::value::Obj;
use yavashark_env::{Error, Object, Value, ValueResult};

pub trait ConstIntoValue {
    fn into_value(self, vm: &impl VM) -> ValueResult;
}

impl ConstIntoValue for ConstValue {
    fn into_value(self, vm: &impl VM) -> ValueResult {
        Ok(match self {
            Self::Null => Value::Null,
            Self::Undefined => Value::Undefined,
            Self::Number(n) => Value::Number(n),
            Self::String(s) => Value::String(s.into()),
            Self::Boolean(b) => Value::Boolean(b),
            Self::Object(obj) => obj.into_value(vm)?,
            Self::Array(array) => array.into_value(vm)?,
            Self::Symbol(ref s) => Value::Symbol(s.into()),
            Self::Function(bp) => {
                let func: RefCell<Box<dyn FunctionCode>> =
                    RefCell::new(Box::new(BytecodeFunction {
                        code: bp.code,
                        is_async: bp.is_async,
                        is_generator: bp.is_generator,
                    }));

                let optim = OptimFunction::new(
                    bp.name.unwrap_or_default(),
                    bp.params,
                    Some(func),
                    vm.get_scope().clone(),
                    vm.get_realm_ref(),
                )?;

                optim.into()
            }
            Self::ArrowFunction(bp) => {
                let func: RefCell<Box<dyn FunctionCode>> =
                    RefCell::new(Box::new(BytecodeArrowFunction {
                        code: bp.code,
                        this: vm.get_this()?,
                        is_async: bp.is_async,
                        is_generator: bp.is_generator,
                    }));

                let optim = OptimFunction::new(
                    String::new(),
                    bp.params,
                    Some(func),
                    vm.get_scope().clone(),
                    vm.get_realm_ref(),
                )?;

                optim.into()
            }
            Self::BigInt(b) => Value::BigInt(b),
            Self::Regex(exp, flags) => {
                RegExp::new_from_str_with_flags(vm.get_realm_ref(), &exp, &flags)?.into()
            }
        })
    }
}

impl ConstIntoValue for ArrayLiteralBlueprint {
    fn into_value(self, vm: &impl VM) -> ValueResult {
        let props = self
            .properties
            .into_iter()
            .map(|v| v.map_or(Ok(Value::Undefined), |v| v.into_value(vm)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Array::with_elements(vm.get_realm_ref(), props)?.into_value())
    }
}

impl ConstIntoValue for ObjectLiteralBlueprint {
    fn into_value(self, vm: &impl VM) -> ValueResult {
        let obj = Object::new(vm.get_realm_ref());

        for (key, value) in self.properties {
            match value {
                DataTypeValue::Get(bp) => {
                    let func: RefCell<Box<dyn FunctionCode>> =
                        RefCell::new(Box::new(BytecodeFunction {
                            code: bp.code,
                            is_async: bp.is_async,
                            is_generator: bp.is_generator,
                        }));

                    let optim = OptimFunction::new(
                        bp.name.unwrap_or_default(),
                        bp.params,
                        Some(func),
                        vm.get_scope().clone(),
                        vm.get_realm_ref(),
                    )?;

                    obj.define_getter(key.into_value(vm)?, optim.into())?;

                    continue;
                }

                DataTypeValue::Set(bp) => {
                    let func: RefCell<Box<dyn FunctionCode>> =
                        RefCell::new(Box::new(BytecodeFunction {
                            code: bp.code,
                            is_async: bp.is_async,
                            is_generator: bp.is_generator,
                        }));

                    let optim = OptimFunction::new(
                        bp.name.unwrap_or_default(),
                        bp.params,
                        Some(func),
                        vm.get_scope().clone(),
                        vm.get_realm_ref(),
                    )?;

                    obj.define_setter(key.into_value(vm)?, optim.into())?;

                    continue;
                }

                _ => {}
            }

            obj.define_property(key.into_value(vm)?, value.into_value(vm)?)?;
        }

        Ok(obj.into())
    }
}

impl ConstIntoValue for DataTypeValue {
    fn into_value(self, vm: &impl VM) -> ValueResult {
        Ok(match self {
            Self::Null => Value::Null,
            Self::Undefined => Value::Undefined,
            Self::Number(n) => Value::Number(n),
            Self::String(s) => Value::String(s.into()),
            Self::Boolean(b) => Value::Boolean(b),
            Self::Object(obj) => obj.into_value(vm)?,
            Self::Array(array) => array.into_value(vm)?,
            Self::Function(bp) => {
                let func: RefCell<Box<dyn FunctionCode>> =
                    RefCell::new(Box::new(BytecodeFunction {
                        code: bp.code,
                        is_async: bp.is_async,
                        is_generator: bp.is_generator,
                    }));

                let optim = OptimFunction::new(
                    bp.name.unwrap_or_default(),
                    bp.params,
                    Some(func),
                    vm.get_scope().clone(),
                    vm.get_realm_ref(),
                )?;

                optim.into()
            }
            Self::Set(bp) => {
                let func: RefCell<Box<dyn FunctionCode>> =
                    RefCell::new(Box::new(BytecodeFunction {
                        code: bp.code,
                        is_async: bp.is_async,
                        is_generator: bp.is_generator,
                    }));

                let optim = OptimFunction::new(
                    bp.name.unwrap_or_default(),
                    bp.params,
                    Some(func),
                    vm.get_scope().clone(),
                    vm.get_realm_ref(),
                )?;

                optim.into()
            }
            Self::Get(bp) => {
                let func: RefCell<Box<dyn FunctionCode>> =
                    RefCell::new(Box::new(BytecodeFunction {
                        code: bp.code,
                        is_async: bp.is_async,
                        is_generator: bp.is_generator,
                    }));

                let optim = OptimFunction::new(
                    bp.name.unwrap_or_default(),
                    bp.params,
                    Some(func),
                    vm.get_scope().clone(),
                    vm.get_realm_ref(),
                )?;

                optim.into()
            }
            Self::ArrowFunction(bp) => {
                let func: RefCell<Box<dyn FunctionCode>> =
                    RefCell::new(Box::new(BytecodeArrowFunction {
                        code: bp.code,
                        this: vm.get_this()?,
                        is_async: bp.is_async,
                        is_generator: bp.is_generator,
                    }));

                let optim = OptimFunction::new(
                    String::new(),
                    bp.params,
                    Some(func),
                    vm.get_scope().clone(),
                    vm.get_realm_ref(),
                )?;

                optim.into()
            }
            Self::BigInt(b) => Value::BigInt(b),
            Self::Regex(exp, flags) => {
                RegExp::new_from_str_with_flags(vm.get_realm_ref(), &exp, &flags)?.into()
            }
            Self::Symbol(ref s) => Value::Symbol(s.into()),
            Self::Acc(_) => vm.acc(),
            Self::Reg(reg) => vm.get_register(reg.0)?,
            Self::Var(var) => vm.get_variable(var.0)?,
            Self::Stack(stack) => vm.get_stack(stack.0).ok_or(Error::new("invalid stack"))?,
        })
    }
}
