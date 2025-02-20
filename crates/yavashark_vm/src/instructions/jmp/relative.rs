use crate::VM;
use yavashark_bytecode::{JmpOffset, VarName};
use yavashark_env::{Res, Value};

pub fn jmp_rel(target: JmpOffset, vm: &mut impl VM) {
    vm.offset_pc(target);
}

pub fn jmp_if_rel(target: JmpOffset, name: VarName, vm: &mut impl VM) -> Res {
    let value = vm.get_variable(name)?;
    if value.is_truthy() {
        vm.offset_pc(target);
    }

    Ok(())
}

pub fn jmp_if_acc_rel(target: JmpOffset, vm: &mut impl VM) -> Res {
    let value = vm.acc();
    if value.is_truthy() {
        vm.offset_pc(target);
    }

    Ok(())
}

pub fn jmp_if_not_rel(target: JmpOffset, name: VarName, vm: &mut impl VM) -> Res {
    let value = vm.get_variable(name)?;
    if !value.is_truthy() {
        vm.offset_pc(target);
    }

    Ok(())
}

pub fn jmp_if_not_acc_rel(target: JmpOffset, vm: &mut impl VM) -> Res {
    let value = vm.acc();
    if !value.is_truthy() {
        vm.offset_pc(target);
    }

    Ok(())
}

pub fn jmp_null_rel(target: JmpOffset, name: VarName, vm: &mut impl VM) -> Res {
    let value = vm.get_variable(name)?;
    if value == Value::Null {
        vm.offset_pc(target);
    }

    Ok(())
}

pub fn jmp_null_acc_rel(target: JmpOffset, vm: &mut impl VM) -> Res {
    let value = vm.acc();
    if value == Value::Null {
        vm.offset_pc(target);
    }

    Ok(())
}

pub fn jmp_undef_rel(target: JmpOffset, name: VarName, vm: &mut impl VM) -> Res {
    let value = vm.get_variable(name)?;
    if value == Value::Undefined {
        vm.offset_pc(target);
    }

    Ok(())
}

pub fn jmp_undef_acc_rel(target: JmpOffset, vm: &mut impl VM) -> Res {
    let value = vm.acc();
    if value == Value::Undefined {
        vm.offset_pc(target);
    }

    Ok(())
}

pub fn jmp_null_undef_rel(target: JmpOffset, name: VarName, vm: &mut impl VM) -> Res {
    let value = vm.get_variable(name)?;
    if value == Value::Null || value == Value::Undefined {
        vm.offset_pc(target);
    }

    Ok(())
}

pub fn jmp_null_undef_acc_rel(target: JmpOffset, vm: &mut impl VM) -> Res {
    let value = vm.acc();
    if value == Value::Null || value == Value::Undefined {
        vm.offset_pc(target);
    }

    Ok(())
}
