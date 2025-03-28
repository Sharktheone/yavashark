use crate::data::{Data, OutputData};
use crate::VM;
use yavashark_env::{ControlFlow, ControlResult, Error, Res, Value};

pub fn nullish_coalescing(
    left: impl Data,
    right: impl Data,
    output: impl OutputData,
    vm: &mut impl VM,
) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    if left.is_nullish() {
        output.set(right, vm)?;
    } else {
        output.set(left, vm)?;
    }

    Ok(())
}

pub fn in_(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    let result = left.contains_key(&right)?.into();

    output.set(result, vm)
}

pub fn instance_of(
    left: impl Data,
    right: impl Data,
    output: impl OutputData,
    vm: &mut impl VM,
) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    let result = left.instance_of(&right, vm.get_realm())?.into();

    output.set(result, vm)
}

pub fn load_member(
    left: impl Data,
    right: impl Data,
    output: impl OutputData,
    vm: &mut impl VM,
) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;

    let result = left.get_property(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn load_var(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let result = data.get(vm)?;

    output.set(result, vm)
}

pub fn type_of(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;
    let result = data.type_of().into();

    output.set(result, vm)
}

pub fn push(data: impl Data, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;

    vm.push(data);

    Ok(())
}

pub fn pop(vm: &mut impl VM) -> Res {
    vm.pop();

    Ok(())
}

pub fn pop_n(n: u32, vm: &mut impl VM) -> Res {
    for _ in 0..n {
        vm.pop().ok_or(Error::new("Stack is empty"))?;
    }

    Ok(())
}

pub fn pop_to(data: impl OutputData, vm: &mut impl VM) -> Res {
    let result = vm.pop().ok_or(Error::new("Stack is empty"))?;

    data.set(result, vm)
}

pub fn move_(from: impl Data, data: impl OutputData, vm: &mut impl VM) -> Res {
    let result = from.get(vm)?;

    data.set(result, vm)
}

pub fn return_(_vm: &mut impl VM) -> ControlResult {
    Err(ControlFlow::Return(Value::Undefined))
}

pub fn return_value(data: impl Data, vm: &mut impl VM) -> ControlResult {
    let result = data.get(vm)?;

    Err(ControlFlow::Return(result))
}

pub fn throw(data: impl Data, vm: &mut impl VM) -> Res {
    let result = data.get(vm)?;

    Err(Error::throw(result))
}

pub fn this(output: impl OutputData, vm: &mut impl VM) -> Res {
    let result = vm.get_this()?;

    output.set(result, vm)
}


pub fn yield_(_data: impl Data, _vm: &impl VM) -> ControlResult {
    // let result = data.get(vm)?;
    // 
    // Err(ControlFlow::Yield(result))
    
    unimplemented!()
}


pub fn await_(_data: impl Data, _vm: &impl VM) -> ControlResult {
    // let result = data.get(vm)?;
    // 
    // Err(ControlFlow::Await(result))
    unimplemented!()
}

pub fn debugger(_vm: &mut impl VM) -> Res {
    dbg!("Set debug point here!");
    Ok(())
}