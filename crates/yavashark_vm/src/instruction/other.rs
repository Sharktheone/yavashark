use crate::data::{Data, OutputData};
use crate::VM;
use yavashark_bytecode::data::{ControlIdx, Label, VarName};
use yavashark_bytecode::JmpAddr;
use yavashark_env::array::Array;
use yavashark_env::builtins::Promise;
use yavashark_env::value::{IntoValue, ObjectOrNull};
use yavashark_env::{
    Class, ClassInstance, ControlFlow, ControlResult, Error, Object, PrivateMember, PropertyKey,
    Realm, Res, Value,
};

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

    let result = left.contains_key(&right, vm.get_realm())?.into();

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

pub fn load_private_member(
    left: impl Data,
    right: impl Data,
    output: impl OutputData,
    vm: &mut impl VM,
) -> Res {
    let base = left.get(vm)?;

    let right = right.get(vm)?;

    let Value::String(name) = right else {
        return Err(Error::ty("Private member name must be a string"));
    };

    let res = get_private_member(vm.get_realm(), base, &name)?.0;

    output.set(res, vm)
}

pub fn store_member(obj: impl Data, prop: impl Data, value: impl Data, vm: &mut impl VM) -> Res {
    let obj = obj.get(vm)?;
    let prop = prop.get(vm)?;
    let value = value.get(vm)?;

    obj.define_property(prop, value, vm.get_realm())?;

    Ok(())
}

pub fn store_private_member(
    obj: impl Data,
    prop: impl Data,
    value: impl Data,
    vm: &mut impl VM,
) -> Res {
    let base = obj.get(vm)?;
    let prop = prop.get(vm)?;
    let value = value.get(vm)?;

    let Value::String(name) = prop else {
        return Err(Error::ty("Private member name must be a string"));
    };

    let obj = base.as_object()?;

    if let Some(class) = obj.downcast::<ClassInstance>() {
        class.update_private_field(&name, value);
        return Ok(());
    }
    if let Some(class) = obj.downcast::<Class>() {
        class.update_private_field(&name, value);
        return Ok(());
    }

    Err(Error::ty(
        "Private member can only be set on class instance or class",
    ))
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

pub const fn return_(_vm: &mut impl VM) -> ControlResult {
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

pub fn yield_star(data: impl Data, vm: &mut impl VM) -> ControlResult {
    let result = data.get(vm)?;

    Err(ControlFlow::YieldStar(result.to_object()?))
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

pub const fn break_(_vm: &mut impl VM) -> ControlResult {
    Err(ControlFlow::Break(None))
}

pub fn break_label(label: Label, vm: &impl VM) -> ControlResult {
    let label = vm.get_label(label)?;

    Err(ControlFlow::Break(Some(label.to_owned())))
}

pub const fn continue_(_vm: &mut impl VM) -> ControlResult {
    Err(ControlFlow::Continue(None))
}

pub fn continue_label(label: Label, vm: &impl VM) -> ControlResult {
    let label = vm.get_label(label)?;

    Err(ControlFlow::Continue(Some(label.to_owned())))
}

pub fn with(data: impl Data, vm: &mut impl VM) -> Res {
    let obj = data.get(vm)?;

    let mut scope = vm.get_scope_mut().clone();

    for (key, value) in obj.properties(vm.get_realm())? {
        let PropertyKey::String(key) = key else {
            continue;
        };

        scope.declare_var(key.to_string(), value, vm.get_realm())?;
    }

    Ok(())
}

pub fn load_super(output: impl OutputData, vm: &mut impl VM) -> Res {
    let this = vm.get_scope().this()?;

    let proto = this.prototype(vm.get_realm())?.to_object()?;

    let sup = proto.prototype(vm.get_realm())?.to_object()?;

    output.set(sup.into(), vm)
}

pub fn load_super_constructor(output: impl OutputData, vm: &mut impl VM) -> Res {
    let this = vm.get_scope().this()?;

    let proto = this.prototype(vm.get_realm())?.to_object()?;

    let sup = proto.prototype(vm.get_realm())?.to_object()?;

    let constructor = sup.get("constructor", vm.get_realm())?;

    output.set(constructor, vm)
}

pub fn enter_try(id: ControlIdx, vm: &mut impl VM) -> Res {
    vm.enter_try(id)
}

pub fn leave_try(vm: &mut impl VM) -> Res {
    vm.leave_try()
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

pub fn iter_collect(iter: impl Data, out: impl OutputData, vm: &mut impl VM) -> Res {
    let iter = iter.get(vm)?;

    let mut elems = Vec::new();

    while let Some(next) = iter.iter_next(vm.get_realm())? {
        elems.push(next);
    }

    let array = Array::with_elements(vm.get_realm(), elems)?;

    out.set(array.into_value(), vm)?;

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

pub fn throw_if_not_object(data: impl Data, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;

    if !data.is_object() {
        return Err(Error::ty("Expected object"));
    }

    Ok(())
}

pub fn begin_spread(cap: usize, vm: &mut impl VM) -> Res {
    vm.begin_spread(cap)
}

pub fn push_spread(elem: impl Data, vm: &mut impl VM) -> Res {
    let elem = elem.get(vm)?;

    vm.push_spread(elem)
}

pub fn end_spread(obj: impl Data, out: impl OutputData, vm: &mut impl VM) -> Res {
    let obj = obj.get(vm)?;

    let rest_obj = vm.end_spread(obj.to_object()?)?;

    out.set(rest_obj.into(), vm)
}

pub fn end_spread_no_output(vm: &mut impl VM) -> Res {
    vm.end_spread_no_output()
}

pub fn to_number(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;

    let num = data.to_number(vm.get_realm())?;

    output.set(num.into(), vm)
}

pub fn to_string(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;

    let string = data.to_string(vm.get_realm())?;

    output.set(string.into(), vm)
}
pub fn to_boolean(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;

    let boolean = data.is_truthy();

    output.set(boolean.into(), vm)
}

pub fn negate(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;

    let num = if let Value::BigInt(b) = data {
        (-(&*b)).into()
    } else {
        Value::Number(-data.to_number(vm.get_realm())?)
    };

    output.set(num, vm)
}

pub fn logical_not(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;

    let boolean = !data.is_truthy();

    output.set(boolean.into(), vm)
}

pub fn bitwise_not(data: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let data = data.get(vm)?;

    let num = if let Value::BigInt(b) = data {
        (!(&*b)).into()
    } else {
        Value::Number(f64::from(!(data.to_number(vm.get_realm())? as i32)))
    };

    output.set(num, vm)
}

pub fn get_new_target(output: impl OutputData, vm: &mut impl VM) -> Res {
    let new_target = vm.get_scope().get_target()?;

    output.set(new_target, vm)
}

pub fn get_import_meta(output: impl OutputData, vm: &mut impl VM) -> Res {
    let obj = Object::with_proto(ObjectOrNull::Null);

    obj.define_property(
        "url".into(),
        vm.get_scope()
            .get_current_path()?
            .to_string_lossy()
            .into_owned()
            .into(),
        vm.get_realm(),
    )?;

    obj.define_property("resolve".into(), Value::Undefined, vm.get_realm())?; //TODO

    output.set(obj.into(), vm)
}

pub(crate) fn resolve_private_member(
    realm: &mut Realm,
    member: PrivateMember,
    base: Value,
) -> Res<(Value, Option<Value>)> {
    match member {
        PrivateMember::Field(value) => Ok((value, None)),
        PrivateMember::Method(func) => Ok((func, Some(base))),
        PrivateMember::Accessor { get, .. } => {
            if let Some(getter) = get {
                let result = getter.call(realm, vec![], base.copy())?;

                Ok((result, None))
            } else {
                Ok((Value::Undefined, None))
            }
        }
    }
}

pub fn get_private_member(
    realm: &mut Realm,
    base: Value,
    name: &str,
) -> Res<(Value, Option<Value>)> {
    let obj = base.as_object()?;

    if let Some(class) = obj.downcast::<ClassInstance>() {
        let member = class
            .get_private_prop(&&name, realm)?
            .ok_or_else(|| ControlFlow::error_type(format!("Private name {name} not found")))?;

        return resolve_private_member(realm, member, base.copy());
    }

    if let Some(class) = obj.downcast::<Class>() {
        let member = class
            .get_private_prop(&name)
            .ok_or_else(|| ControlFlow::error_type(format!("Private name {name} not found")))?;

        return resolve_private_member(realm, member, base.copy());
    }

    return Err(Error::ty_error(format!(
        "Private name {name} can only be used in class"
    )));
}


pub fn import_dynamic(
    name: VarName,
    path: VarName,
    output: impl OutputData,
    vm: &mut impl VM,
) -> ControlResult {
    Err(Error::new("Dynamic import is not supported in VM yet").into())
}