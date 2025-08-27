use crate::data::{Data, OutputData};
use crate::VM;
use yavashark_bytecode::data::{ControlIdx, Label};
use yavashark_bytecode::JmpAddr;
use yavashark_env::builtins::Promise;
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

    let result = left.get_property_opt(&right, vm.get_realm())?;

    output.set(result.unwrap_or(Value::Undefined), vm)
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

pub fn yield_(data: impl Data, vm: &mut impl VM) -> ControlResult {
    let result = data.get(vm)?;

    Err(ControlFlow::Yield(result))
}

pub const fn yield_undefined(_: &impl VM) -> ControlResult {
    Err(ControlFlow::Yield(Value::Undefined))
}

pub fn await_(data: impl Data, out: impl OutputData, vm: &mut impl VM) -> ControlResult {
    let result = data.get(vm)?;

    match result {
        Value::Object(obj) if obj.downcast::<Promise>().is_some() => {
            vm.set_continue_storage(out);
            return Err(ControlFlow::Await(obj));
        }

        _ => out.set(result, vm)?,
    }

    Ok(())
}

pub fn await_no_output(data: impl Data, vm: &mut impl VM) -> ControlResult {
    let result = data.get(vm)?;

    if let Value::Object(obj) = result {
        if obj.downcast::<Promise>().is_some() {
            return Err(ControlFlow::Await(obj));
        }
    }

    Ok(())
}

pub fn debugger(_vm: &mut impl VM) -> Res {
    dbg!("Set debug point here!");
    Ok(())
}

pub fn break_(_vm: &mut impl VM) -> ControlResult {
    Err(ControlFlow::Break(None))
}

pub fn break_label(label: Label, vm: &impl VM) -> ControlResult {
    let label = vm.get_label(label)?;

    Err(ControlFlow::Break(Some(label.to_owned())))
}

pub fn continue_(_vm: &mut impl VM) -> ControlResult {
    Err(ControlFlow::Continue(None))
}

pub fn continue_label(label: Label, vm: &impl VM) -> ControlResult {
    let label = vm.get_label(label)?;

    Err(ControlFlow::Continue(Some(label.to_owned())))
}

pub fn with(data: impl Data, vm: &mut impl VM) -> Res {
    let obj = data.get(vm)?;

    let scope = vm.get_scope_mut();

    for (key, value) in obj.properties()? {
        let Value::String(key) = key else {
            continue;
        };

        scope.declare_var(key.to_string(), value)?;
    }

    Ok(())
}

pub fn load_super(output: impl OutputData, vm: &mut impl VM) -> Res {
    let this = vm.get_scope().this()?;

    let proto = this.prototype(vm.get_realm())?;
    let sup = proto.prototype(vm.get_realm())?;

    output.set(sup, vm)
}

pub fn load_super_constructor(output: impl OutputData, vm: &mut impl VM) -> Res {
    let this = vm.get_scope().this()?;

    let proto = this.prototype(vm.get_realm())?;
    let sup = proto.prototype(vm.get_realm())?;

    let constructor = sup.as_object()?.constructor()?;

    let constructor = constructor.resolve(proto.copy(), vm.get_realm())?;

    output.set(constructor, vm)
}

pub fn enter_try(id: ControlIdx, vm: &mut impl VM) -> Res {
    vm.enter_try(id)
}

pub fn leave_try(vm: &mut impl VM) -> Res {
    vm.leave_try()
}

pub fn pat_begin_rest(_len: usize, _vm: &mut impl VM) -> Res {
    todo!()
}
pub fn pat_void_next(_vm: &mut impl VM) -> Res {
    todo!()
}

pub fn pat_move_let(_: impl Data, _: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}
pub fn pat_move_const(_: impl Data, _: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}
pub fn pat_move_var(_: impl Data, _: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}

pub fn pat_rest_let(_: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}
pub fn pat_rest_const(_: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}
pub fn pat_rest_var(_: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}

pub fn pat_array_move_let(_: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}
pub fn pat_array_move_const(_: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}
pub fn pat_array_move_var(_: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}

pub fn pat_array_rest_let(_: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}

pub fn pat_array_rest_const(_: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}

pub fn pat_array_rest_var(_: impl Data, _vm: &mut impl VM) -> Res {
    todo!()
}

pub fn push_iter(iter: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let iter = iter.get(vm)?;

    let iter = iter.get_iter(vm.get_realm())?;

    output.set(iter, vm)?;

    Ok(())
}

pub fn iter_next(iter: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let iter = iter.get(vm)?;

    let next = iter.iter_next(vm.get_realm())?;

    output.set(next.unwrap_or(Value::Undefined), vm)
}

pub fn iter_next_no_output(iter: impl Data, vm: &mut impl VM) -> Res {
    let iter = iter.get(vm)?;

    iter.iter_next_no_out(vm.get_realm())
}

pub fn iter_next_jmp(
    iter: impl Data,
    addr: JmpAddr,
    output: impl OutputData,
    vm: &mut impl VM,
) -> Res {
    let iter = iter.get(vm)?;

    let next = iter.iter_next(vm.get_realm())?;

    if let Some(next) = next {
        output.set(next, vm)?;
    } else {
        vm.set_pc(addr);
    }

    Ok(())
}

pub fn iter_next_no_output_jmp(iter: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let iter = iter.get(vm)?;

    let finished = iter.iter_next_is_finished(vm.get_realm())?;

    if finished {
        vm.set_pc(addr);
    }

    Ok(())
}

pub fn push_async_iter(iter: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let iter = iter.get(vm)?;

    let iter = iter.get_async_iter(vm.get_realm())?;

    output.set(iter, vm)?;

    Ok(())
}

pub fn async_iter_poll_next(
    iter: impl Data,
    output: impl OutputData,
    vm: &mut impl VM,
) -> ControlResult {
    let iter = iter.get(vm)?;

    let next = iter.async_iter_next(vm.get_realm())?;

    match next {
        Value::Object(obj) if obj.downcast::<Promise>().is_some() => {
            vm.set_continue_storage(output);
            return Err(ControlFlow::Await(obj));
        }

        _ => output.set(next, vm)?,
    }

    Ok(())
}

pub fn async_iter_next(next: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let next = next.get(vm)?;

    let next = next.iter_res(vm.get_realm())?;

    output.set(next.unwrap_or(Value::Undefined), vm)
}

pub fn async_iter_next_jmp(
    next: impl Data,
    addr: JmpAddr,
    output: impl OutputData,
    vm: &mut impl VM,
) -> Res {
    let next = next.get(vm)?;

    let next = next.iter_res(vm.get_realm())?;

    if let Some(next) = next {
        output.set(next, vm)?;
    } else {
        vm.set_pc(addr);
    }

    Ok(())
}

pub fn async_iter_next_no_output_jmp(next: impl Data, addr: JmpAddr, vm: &mut impl VM) -> Res {
    let next = next.get(vm)?;

    let finished = next.iter_done(vm.get_realm())?;

    if finished {
        vm.set_pc(addr);
    }

    Ok(())
}
