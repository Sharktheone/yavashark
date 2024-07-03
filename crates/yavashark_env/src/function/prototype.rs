#![allow(clippy::needless_pass_by_value)]

use yavashark_value::Obj;

use crate::context::Context;
use crate::object::Object;
use crate::{NativeFunction, Res, Value, ValueResult, Variable};

#[derive(Debug)]
pub struct FunctionPrototype {
    pub object: Object,
    pub apply: Variable,
    pub bind: Variable,
    pub call: Variable,
    pub constructor: Variable,
    pub length: Variable,
    pub name: Variable,
}

impl FunctionPrototype {
    #[must_use]
    pub fn new(obj: Value) -> Self {
        Self {
            object: Object::raw_with_proto(obj),
            apply: Value::Undefined.into(),
            bind: Value::Undefined.into(),
            call: Value::Undefined.into(),
            constructor: Value::Undefined.into(),
            length: Value::Number(0.0).into(),
            name: Value::String("Function".to_string()).into(),
        }
    }

    pub fn initialize(&mut self, func: Value) {
        self.apply = NativeFunction::with_proto("apply", apply, func.copy()).into();
        self.bind = NativeFunction::with_proto("bind", bind, func.copy()).into();
        self.call = NativeFunction::with_proto("call", call, func.copy()).into();
        self.constructor = NativeFunction::with_proto("Function", constructor, func.copy()).into();
    }
}

#[allow(unused)]
fn apply(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

#[allow(unused)]
fn bind(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

#[allow(unused)]
fn call(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

#[allow(unused)]
fn constructor(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

impl Obj<Context> for FunctionPrototype {
    fn define_property(&mut self, name: Value, value: Value) {
        if let Value::String(name) = &name {
            match name.as_str() {
                "apply" => {
                    self.apply = value.into();
                    return;
                }
                "bind" => {
                    self.bind = value.into();
                    return;
                }
                "call" => {
                    self.call = value.into();
                    return;
                }
                "constructor" => {
                    self.constructor = value.into();
                    return;
                }
                "length" => {
                    self.length = value.into();
                    return;
                }
                "name" => {
                    self.name = value.into();
                    return;
                }
                _ => {}
            }
        }

        self.object.define_property(name, value);
    }

    fn define_variable(&mut self, name: Value, value: Variable) {
        if let Value::String(name) = &name {
            match name.as_str() {
                "apply" => {
                    self.apply = value;
                    return;
                }
                "bind" => {
                    self.bind = value;
                    return;
                }
                "call" => {
                    self.call = value;
                    return;
                }
                "constructor" => {
                    self.constructor = value;
                    return;
                }
                "length" => {
                    self.length = value;
                    return;
                }
                "name" => {
                    self.name = value;
                    return;
                }
                _ => {}
            }
        }

        self.object.define_variable(name, value);
    }

    fn resolve_property(&self, name: &Value) -> Option<Value> {
        if let Value::String(name) = name {
            match name.as_str() {
                "apply" => return Some(self.apply.value.copy()),
                "bind" => return Some(self.bind.value.copy()),
                "call" => return Some(self.call.value.copy()),
                "constructor" => return Some(self.constructor.value.copy()),
                "length" => return Some(self.length.value.copy()),
                "name" => return Some(self.name.value.copy()),
                _ => {}
            }
        }

        self.object.resolve_property(name)
    }

    fn get_property(&self, name: &Value) -> Option<&Value> {
        if let Value::String(name) = name {
            match name.as_str() {
                "apply" => return Some(&self.apply.value),
                "bind" => return Some(&self.bind.value),
                "call" => return Some(&self.call.value),
                "constructor" => return Some(&self.constructor.value),
                "length" => return Some(&self.length.value),
                "name" => return Some(&self.name.value),
                _ => {}
            }
        }

        self.object.get_property(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        if let Value::String(name) = name {
            match name.as_str() {
                "apply" => return Some(&mut self.apply.value),
                "bind" => return Some(&mut self.bind.value),
                "call" => return Some(&mut self.call.value),
                "constructor" => return Some(&mut self.constructor.value),
                "length" => return Some(&mut self.length.value),
                "name" => return Some(&mut self.name.value),
                _ => {}
            }
        }

        self.object.get_property_mut(name)
    }


    fn define_getter(&mut self, name: Value, value: Value) -> Res {
        self.object.define_getter(name, value)
    }
    
    fn define_setter(&mut self, name: Value, value: Value) -> Res {
        self.object.define_setter(name, value)
    }

    fn delete_property(&mut self, name: &Value) -> Option<Value> {
        if let Value::String(name) = name {
            match name.as_str() {
                "apply" => {
                    let old = self.apply.value.copy();
                    self.apply = Value::Undefined.into();
                    return Some(old);
                }
                "bind" => {
                    let old = self.bind.value.copy();
                    self.bind = Value::Undefined.into();
                    return Some(old);
                }
                "call" => {
                    let old = self.call.value.copy();
                    self.call = Value::Undefined.into();
                    return Some(old);
                }
                "constructor" => {
                    let old = self.constructor.value.copy();
                    self.constructor = Value::Undefined.into();
                    return Some(old);
                }
                "length" => {
                    let old = self.length.value.copy();
                    self.length = Value::Undefined.into();
                    return Some(old);
                }
                "name" => {
                    let old = self.name.value.copy();
                    self.name = Value::Undefined.into();
                    return Some(old);
                }
                _ => {}
            }
        }

        self.object.delete_property(name)
    }

    fn contains_key(&self, name: &Value) -> bool {
        if let Value::String(name) = name {
            match name.as_str() {
                "apply" | "bind" | "call" | "constructor" | "length" | "name" => return true,
                _ => {}
            }
        }

        self.object.contains_key(name)
    }

    fn name(&self) -> String {
        self.name.value.to_string()
    }

    fn to_string(&self) -> String {
        "Function Prototype".to_string()
    }

    fn properties(&self) -> Vec<(Value, Value)> {
        let mut props = self.object.properties();
        props.push((Value::String("apply".to_string()), self.apply.value.copy()));
        props.push((Value::String("bind".to_string()), self.bind.value.copy()));
        props.push((Value::String("call".to_string()), self.call.value.copy()));
        props.push((
            Value::String("constructor".to_string()),
            self.constructor.value.copy(),
        ));
        props.push((
            Value::String("length".to_string()),
            self.length.value.copy(),
        ));
        props.push((Value::String("name".to_string()), self.name.value.copy()));
        props
    }

    fn keys(&self) -> Vec<Value> {
        let mut keys = self.object.keys();
        keys.push(Value::String("apply".to_string()));
        keys.push(Value::String("bind".to_string()));
        keys.push(Value::String("call".to_string()));
        keys.push(Value::String("constructor".to_string()));
        keys.push(Value::String("length".to_string()));
        keys.push(Value::String("name".to_string()));
        keys
    }

    fn values(&self) -> Vec<Value> {
        let mut values = self.object.values();
        values.push(self.apply.value.copy());
        values.push(self.bind.value.copy());
        values.push(self.call.value.copy());
        values.push(self.constructor.value.copy());
        values.push(self.length.value.copy());
        values.push(self.name.value.copy());
        values
    }

    fn get_array_or_done(&self, index: usize) -> (bool, Option<Value>) {
        self.object.get_array_or_done(index)
    }

    fn clear_values(&mut self) {
        self.object.clear_values();
        self.apply = Value::Undefined.into();
        self.bind = Value::Undefined.into();
        self.call = Value::Undefined.into();
        self.constructor = Value::Undefined.into();
        self.length = Value::Number(0.0).into();
        self.name = Value::String("Function".to_string()).into();
    }

    fn prototype(&self) -> Value {
        self.object.prototype()
    }
}
