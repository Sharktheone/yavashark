#![allow(clippy::needless_pass_by_value)]

use std::cell::RefCell;
use yavashark_value::{MutObj, Obj};

use crate::array::Array;
use crate::function::bound::BoundFunction;
use crate::realm::Realm;
use crate::{
    Error, MutObject, NativeFunction, ObjectHandle, ObjectProperty, Res, Value, ValueResult,
    Variable,
};

#[derive(Debug)]
struct MutableFunctionPrototype {
    pub object: MutObject,
    pub apply: ObjectProperty,
    pub bind: ObjectProperty,
    pub call: ObjectProperty,
    pub constructor: ObjectProperty,
    pub length: ObjectProperty,
    pub name: ObjectProperty,
    pub to_string: ObjectProperty,
}

#[derive(Debug)]
pub struct FunctionPrototype {
    inner: RefCell<MutableFunctionPrototype>,
}

impl FunctionPrototype {
    #[must_use]
    pub fn new(obj: Value) -> Self {
        Self {
            inner: RefCell::new(MutableFunctionPrototype {
                object: MutObject::with_proto(obj),
                apply: Value::Undefined.into(),
                bind: Value::Undefined.into(),
                call: Value::Undefined.into(),
                constructor: Value::Undefined.into(),
                length: Value::Number(0.0).into(),
                name: Value::String("Function".to_string()).into(),
                to_string: Value::Undefined.into(),
            }),
        }
    }

    pub fn initialize(&self, func: Value) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.apply = NativeFunction::with_proto("apply", apply, func.copy()).into();
        this.bind = NativeFunction::with_proto("bind", bind, func.copy()).into();
        this.call = NativeFunction::with_proto("call", call, func.copy()).into();
        this.constructor = NativeFunction::with_proto("Function", constructor, func.copy()).into();
        this.to_string = NativeFunction::with_proto("toString", to_string, func.copy()).into();

        this.constructor
            .value
            .define_property("prototype".into(), func)
    }
}

#[allow(unused)]
fn apply(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Err(Error::new("Not enough arguments"));
    }

    let new_this = &args[0];

    let args = if let Some(arr) = args.get(1) {
        let array = Array::from_array_like(realm, arr.copy())?;

        array.as_vec()?
    } else {
        vec![]
    };

    this.call(realm, args, new_this.copy())
}

#[allow(unused)]
fn bind(mut args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return BoundFunction::new(this, Value::Undefined, vec![], realm);
    }

    BoundFunction::new(this, args.remove(0), args, realm)
}

#[allow(unused)]
fn call(mut args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    let new_this = if args.is_empty() {
        Value::Undefined
    } else {
        args.remove(0)
    };

    this.call(realm, args, new_this)
}

#[allow(unused)]
fn constructor(mut args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    let Some(body) = args.pop() else {
        return Ok(NativeFunction::new("anonymous", |_, _, _| Ok(Value::Undefined), realm).into());
    };

    let mut buf = "function anonymous(".to_owned();

    for (i, arg) in args.iter().enumerate() {
        if i != 0 {
            buf.push(',');
        }

        buf.push_str(&arg.to_string(realm)?);
    }

    buf.push_str(") { ");

    buf.push_str(&body.to_string(realm)?);

    buf.push_str(" }");

    buf.push_str("anonymous");

    let Some(eval) = realm.intrinsics.eval.clone() else {
        return Err(Error::new("eval is not defined"));
    };

    eval.call(realm, vec![Value::String(buf)], Value::Undefined)
}

fn to_string(_args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Ok(this.to_string(realm)?.into())
}

impl Obj<Realm> for FunctionPrototype {
    fn define_property(&self, name: Value, value: Value) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        if let Value::String(name) = &name {
            match name.as_str() {
                "apply" => {
                    this.apply = value.into();
                    return Ok(());
                }
                "bind" => {
                    this.bind = value.into();
                    return Ok(());
                }
                "call" => {
                    this.call = value.into();
                    return Ok(());
                }
                "constructor" => {
                    this.constructor = value.into();
                    return Ok(());
                }
                "length" => {
                    this.length = value.into();
                    return Ok(());
                }
                "name" => {
                    this.name = value.into();
                    return Ok(());
                }
                "toString" => {
                    this.to_string = value.into();
                    return Ok(());
                }

                _ => {}
            }
        }

        this.object.define_property(name, value)
    }

    fn define_variable(&self, name: Value, value: Variable) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        if let Value::String(name) = &name {
            match name.as_str() {
                "apply" => {
                    this.apply = value.into();
                    return Ok(());
                }
                "bind" => {
                    this.bind = value.into();
                    return Ok(());
                }
                "call" => {
                    this.call = value.into();
                    return Ok(());
                }
                "constructor" => {
                    this.constructor = value.into();
                    return Ok(());
                }
                "length" => {
                    this.length = value.into();
                    return Ok(());
                }
                "name" => {
                    this.name = value.into();
                    return Ok(());
                }
                "toString" => {
                    this.to_string = value.into();
                    return Ok(());
                }
                _ => {}
            }
        }

        this.object.define_variable(name, value)
    }

    fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        let this = self.inner.try_borrow()?;

        if let Value::String(name) = name {
            match name.as_str() {
                "apply" => return Ok(Some(this.apply.clone())),
                "bind" => return Ok(Some(this.bind.clone())),
                "call" => return Ok(Some(this.call.clone())),
                "constructor" => return Ok(Some(this.constructor.clone())),
                "length" => return Ok(Some(this.length.clone())),
                "name" => return Ok(Some(this.name.clone())),
                "toString" => return Ok(Some(this.to_string.clone())),
                _ => {}
            }
        }

        this.object.resolve_property(name)
    }

    fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        let this = self.inner.try_borrow()?;

        if let Value::String(name) = name {
            match name.as_str() {
                "apply" => return Ok(Some(this.apply.copy())),
                "bind" => return Ok(Some(this.bind.copy())),
                "call" => return Ok(Some(this.call.copy())),
                "constructor" => return Ok(Some(this.constructor.copy())),
                "length" => return Ok(Some(this.length.copy())),
                "name" => return Ok(Some(this.name.copy())),
                "toString" => return Ok(Some(this.to_string.copy())),
                _ => {}
            }
        }

        this.object.get_property(name).map(|v| v.map(|v| v.copy()))
    }

    fn define_getter(&self, name: Value, value: Value) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.define_getter(name, value)
    }

    fn define_setter(&self, name: Value, value: Value) -> Res {
        let mut this = self.inner.try_borrow_mut()?;
        this.object.define_setter(name, value)
    }

    fn get_getter(&self, name: &Value) -> Res<Option<Value>, Error> {
        let this = self.inner.try_borrow()?;
        this.object.get_getter(name)
    }

    fn get_setter(&self, name: &Value) -> Res<Option<Value>> {
        let this = self.inner.try_borrow()?;
        this.object.get_setter(name)
    }

    fn delete_property(&self, name: &Value) -> Res<Option<Value>> {
        let mut this = self.inner.try_borrow_mut()?;

        if let Value::String(name) = name {
            match name.as_str() {
                "apply" => {
                    let old = this.apply.value.copy();
                    this.apply = Value::Undefined.into();
                    return Ok(Some(old));
                }
                "bind" => {
                    let old = this.bind.value.copy();
                    this.bind = Value::Undefined.into();
                    return Ok(Some(old));
                }
                "call" => {
                    let old = this.call.value.copy();
                    this.call = Value::Undefined.into();
                    return Ok(Some(old));
                }
                "constructor" => {
                    let old = this.constructor.value.copy();
                    this.constructor = Value::Undefined.into();
                    return Ok(Some(old));
                }
                "length" => {
                    let old = this.length.value.copy();
                    this.length = Value::Undefined.into();
                    return Ok(Some(old));
                }
                "name" => {
                    let old = this.name.value.copy();
                    this.name = Value::Undefined.into();
                    return Ok(Some(old));
                }
                "toString" => {
                    let old = this.to_string.value.copy();
                    this.to_string = Value::Undefined.into();
                    return Ok(Some(old));
                }
                _ => {}
            }
        }

        this.object.delete_property(name)
    }

    fn contains_key(&self, name: &Value) -> Res<bool> {
        if let Value::String(name) = name {
            match name.as_str() {
                "apply" | "bind" | "call" | "constructor" | "length" | "name" | "toString" => {
                    return Ok(true)
                }
                _ => {}
            }
        }

        let this = self.inner.try_borrow()?;

        this.object.contains_key(name)
    }

    fn name(&self) -> String {
        "FunctionPrototype".to_string()
    }

    fn to_string(&self, _realm: &mut Realm) -> Res<String, Error> {
        Ok("function () { [Native code] } ".to_string())
    }

    fn to_string_internal(&self) -> Res<String> {
        Ok("function () { [Native code <Function Prototype>] } ".to_string())
    }

    fn properties(&self) -> Res<Vec<(Value, Value)>> {
        let this = self.inner.try_borrow()?;

        let mut props = this.object.properties()?;
        props.push((Value::String("apply".to_string()), this.apply.value.copy()));
        props.push((Value::String("bind".to_string()), this.bind.value.copy()));
        props.push((Value::String("call".to_string()), this.call.value.copy()));
        props.push((
            Value::String("constructor".to_string()),
            this.constructor.value.copy(),
        ));
        props.push((
            Value::String("length".to_string()),
            this.length.value.copy(),
        ));
        props.push((Value::String("name".to_string()), this.name.value.copy()));
        props.push((
            Value::String("toString".to_string()),
            this.to_string.value.copy(),
        ));

        Ok(props)
    }

    fn keys(&self) -> Res<Vec<Value>> {
        let this = self.inner.try_borrow()?;

        let mut keys = this.object.keys()?;
        keys.push(Value::String("apply".to_string()));
        keys.push(Value::String("bind".to_string()));
        keys.push(Value::String("call".to_string()));
        keys.push(Value::String("constructor".to_string()));
        keys.push(Value::String("length".to_string()));
        keys.push(Value::String("name".to_string()));
        keys.push(Value::String("toString".to_string()));

        Ok(keys)
    }

    fn values(&self) -> Res<Vec<Value>> {
        let this = self.inner.try_borrow()?;

        let mut values = this.object.values()?;
        values.push(this.apply.value.copy());
        values.push(this.bind.value.copy());
        values.push(this.call.value.copy());
        values.push(this.constructor.value.copy());
        values.push(this.length.value.copy());
        values.push(this.name.value.copy());
        values.push(this.to_string.value.copy());

        Ok(values)
    }

    fn get_array_or_done(&self, index: usize) -> Res<(bool, Option<Value>)> {
        let this = self.inner.try_borrow()?;

        this.object.get_array_or_done(index)
    }

    fn clear_values(&self) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.clear_values()?;
        this.apply = Value::Undefined.into();
        this.bind = Value::Undefined.into();
        this.call = Value::Undefined.into();
        this.constructor = Value::Undefined.into();
        this.length = Value::Number(0.0).into();
        this.name = Value::String("Function".to_string()).into();
        this.to_string = Value::Undefined.into();

        Ok(())
    }

    fn prototype(&self) -> Res<ObjectProperty> {
        let this = self.inner.try_borrow()?;

        this.object.prototype()
    }
}
