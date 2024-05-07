use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use yavashark_value::{Func, Obj};

use crate::context::Context;
use crate::object::Prototype;
use crate::{NativeFunction, Value, ValueResult, Variable};

#[derive(Debug)]
// #[object(prototype, direct(apply, bind, call, length, name), constructor)]
pub struct FunctionPrototype {
    pub properties: HashMap<Value, Variable>,
    pub parent: Rc<RefCell<Prototype>>,
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
            properties: HashMap::new(),
            parent: Rc::new(RefCell::new(Prototype::new())),
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

fn apply(args: Vec<Value>, this: Value) -> ValueResult {
    todo!()
}

fn bind(args: Vec<Value>, this: Value) -> ValueResult {
    todo!()
}

fn call(args: Vec<Value>, this: Value) -> ValueResult {
    todo!()
}

fn constructor(args: Vec<Value>, this: Value) -> ValueResult {
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

        self.properties.insert(name, value.into());
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

        self.properties.insert(name, value);
    }
    
    fn resolve_property(&self, name: &Value) -> Option<Value> {
        self.properties.get(name).map(|v| v.copy())
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

        Some(&self.properties.get(name)?.value)
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

        Some(&mut self.properties.get_mut(name)?.value)
    }

    fn contains_key(&self, name: &Value) -> bool {
        if let Value::String(name) = name {
            match name.as_str() {
                "apply" => return true,
                "bind" => return true,
                "call" => return true,
                "constructor" => return true,
                "length" => return true,
                "name" => return true,
                _ => {}
            }
        }

        self.properties.contains_key(name)
    }

    fn name(&self) -> String {
        self.name.to_string()
    }

    fn to_string(&self) -> String {
        "function() { [Native code] }".to_string()
    }

    fn properties(&self) -> Vec<(Value, Value)> {
        self.properties
            .iter()
            .map(|(k, v)| (k.copy(), v.copy()))
            .collect()
    }

    fn keys(&self) -> Vec<Value> {
        self.properties.keys().map(|v| v.copy()).collect()
    }

    fn values(&self) -> Vec<Value> {
        self.properties.values().map(|v| v.copy()).collect()
    }
}

impl Func<Context> for FunctionPrototype {
    fn call(&mut self, _ctx: &mut Context, args: Vec<Value>, this: Value) -> ValueResult {
        todo!()
    }
}
