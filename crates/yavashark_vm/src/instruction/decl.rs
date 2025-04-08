use crate::data::Data;
use crate::VM;
use yavashark_bytecode::data::VarName;
use yavashark_env::{Error, Res, Value};

pub fn decl_const(data: impl Data, var: VarName, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;
    let name = vm
        .var_name(var.0)
        .ok_or(Error::new("Variable not found"))?
        .to_owned();

    vm.get_scope_mut().declare_read_only_var(name, data)
}

pub fn decl_var(data: impl Data, var: VarName, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;
    let name = vm
        .var_name(var.0)
        .ok_or(Error::new("Variable not found"))?
        .to_owned();

    vm.get_scope_mut().declare_global_var(name, data)
}

pub fn decl_empty_var(var: VarName, vm: &mut impl VM) -> Res {
    let name = vm
        .var_name(var.0)
        .ok_or(Error::new("Variable not found"))?
        .to_owned();

    vm.get_scope_mut()
        .declare_global_var(name, Value::Undefined)
}

pub fn decl_let(data: impl Data, var: VarName, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;
    let name = vm
        .var_name(var.0)
        .ok_or(Error::new("Variable not found"))?
        .to_owned();

    vm.get_scope_mut().declare_var(name, data)
}

pub fn decl_empty_let(var: VarName, vm: &mut impl VM) -> Res {
    let name = vm
        .var_name(var.0)
        .ok_or(Error::new("Variable not found"))?
        .to_owned();

    vm.get_scope_mut().declare_var(name, Value::Undefined)
}
