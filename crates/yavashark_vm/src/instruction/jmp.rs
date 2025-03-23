use yavashark_bytecode::JmpAddr;
use yavashark_env::{Res, Value};
use crate::data::Data;
use crate::VM;

pub fn jmp(addr: JmpAddr, vm: &mut impl VM) -> Res {
    vm.set_pc(addr);
    
    Ok(())
}

pub fn jmp_if(condition: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;
    
    if condition.is_truthy() {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_not(condition: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;
    
    if !condition.is_truthy() {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_null(condition: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;
    
    if condition == Value::Null {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_not_null(condition: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;
    
    if condition != Value::Null {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_undefined(condition: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;
    
    if condition == Value::Undefined {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_not_undefined(condition: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;
    
    if condition != Value::Undefined {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_nullish(condition: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;
    
    if condition.is_nullish() {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_not_nullish(condition: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let condition = condition.get(vm)?;
    
    if !condition.is_nullish() {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_eq(left: impl Data, right: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    
    if left.normal_eq(&right) {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_ne(left: impl Data, right: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    
    if !left.normal_eq(&right) {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_strict_eq(left: impl Data, right: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    
    if left == right {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_strict_ne(left: impl Data, right: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    
    if left != right {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_lt(left: impl Data, right: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    
    if left < right {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_lt_eq(left: impl Data, right: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left <= right {
        vm.set_pc(addr);
    }

    Ok(())
}

pub fn jmp_if_gt(left: impl Data, right: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    
    if left > right {
        vm.set_pc(addr);
    }
    
    Ok(())
}

pub fn jmp_if_gt_eq(left: impl Data, right: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    
    if left >= right {
        vm.set_pc(addr);
    }
    
    Ok(())
}