#![allow(clippy::needless_pass_by_value)]

use yavashark_value::{Func, Obj};

use crate::{NativeFunction, Value, ValueResult, Variable};
use crate::context::Context;
use crate::object::Object;

#[derive(Debug)]
// #[object(prototype, direct(apply, bind, call, length, name), constructor)]
pub struct FunctionPrototype {
    pub object: Object,
    pub apply: Variable,
    pub bind: Variable,
    pub call: Variable,
    pub constructor: Variable,
    pub length: Variable,
    pub name: Variable,
}

// #[properties]
impl FunctionPrototype {
    // #[call]
    // #[constructor]
    // #[attributes(writable = false, enumerable = false, configurable = false)]
    // #[name("Function")]
    pub fn new(obj: &Value) -> Self {
        let mut this = Self {
            object: Object::raw_with_proto(obj.copy()),
            apply: Value::Undefined.into(),
            bind: Value::Undefined.into(),
            call: Value::Undefined.into(),
            constructor: Value::Undefined.into(),
            length: Value::Number(0.0).into(),
            name: Value::String("Function".to_string()).into(),
        };
        this.apply = NativeFunction::with_proto("apply", apply, obj.copy()).into();
        this.bind = NativeFunction::with_proto("bind", bind, obj.copy()).into();
        this.call = NativeFunction::with_proto("call", call, obj.copy()).into();
        this.constructor = NativeFunction::with_proto("Function", constructor, obj.copy()).into();

        this
    }
}

fn apply(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

fn bind(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

fn call(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
    todo!()
}

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

    fn contains_key(&self, name: &Value) -> bool {
        if let Value::String(name) = name {
            match name.as_str() {
                "apply"
                | "bind"
                | "call"
                | "constructor"
                | "length"
                | "name" => return true,
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
        props.push((Value::String("constructor".to_string()), self.constructor.value.copy()));
        props.push((Value::String("length".to_string()), self.length.value.copy()));
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
}

impl Func<Context> for FunctionPrototype {
    fn call(&mut self, _ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
        todo!()
    }
}
