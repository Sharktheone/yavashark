use crate::data::Data;
use crate::VM;
use yavashark_bytecode::JmpOffset;
use yavashark_env::{Res, Value};

pub fn jmp_rel(addr: JmpOffset, vm: &mut impl VM) -> Res {
    vm.offset_pc(addr);

    Ok(())
}

pub fn jmp_if_rel(condition: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;

    if condition.is_truthy() {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_not_rel(condition: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;

    if !condition.is_truthy() {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_null_rel(condition: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;

    if condition == Value::Null {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_not_null_rel(condition: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;

    if condition != Value::Null {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_undefined_rel(condition: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;

    if condition == Value::Undefined {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_not_undefined_rel(condition: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;

    if condition != Value::Undefined {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_nullish_rel(condition: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;

    if condition.is_nullish() {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_not_nullish_rel(condition: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;

    if !condition.is_nullish() {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_eq_rel(left: impl Data, right: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left.normal_eq(&right, vm.get_realm())? {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_ne_rel(left: impl Data, right: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if !left.normal_eq(&right, vm.get_realm())? {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_strict_eq_rel(
    left: impl Data,
    right: impl Data,
    addr: JmpOffset,
    vm: &mut impl VM,
) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left == right {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_strict_ne_rel(
    left: impl Data,
    right: impl Data,
    addr: JmpOffset,
    vm: &mut impl VM,
) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left != right {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_lt_rel(left: impl Data, right: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left < right {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_lt_eq_rel(
    left: impl Data,
    right: impl Data,
    addr: JmpOffset,
    vm: &mut impl VM,
) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left <= right {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_gt_rel(left: impl Data, right: impl Data, addr: JmpOffset, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left > right {
        vm.offset_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_gt_eq_rel(
    left: impl Data,
    right: impl Data,
    addr: JmpOffset,
    vm: &mut impl VM,
) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left >= right {
        vm.offset_pc(addr);
    }

    Ok(())
}
