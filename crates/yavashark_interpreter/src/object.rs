use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::Variable;
pub use prototype::*;
use yavashark_value::Obj;

use crate::context::Context;
use crate::Value;

pub mod array;
mod prototype;

#[derive(Debug)]
pub struct Object {
    pub properties: HashMap<Value, Variable>,
    pub array: Vec<(usize, Variable)>,
    pub prototype: Variable,
}

impl Object {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(context: &mut Context) -> crate::ObjectHandle {
        let prototype = context.obj_prototype.clone().into();

        let this: Box<dyn Obj<Context>> = Box::new(Self {
            properties: HashMap::new(),
            prototype,
            array: Vec::new(),
        });

        this.into()
    }

    pub fn with_proto(proto: Value) -> crate::ObjectHandle {
        let this: Box<dyn Obj<Context>> = Box::new(Self {
            properties: HashMap::new(),
            prototype: proto.into(),
            array: Vec::new(),
        });

        this.into()
    }

    pub fn raw(context: &mut Context) -> Self {
        let prototype = context.obj_prototype.clone().into();

        Self {
            properties: HashMap::new(),
            prototype,
            array: Vec::new(),
        }
    }

    pub fn raw_with_proto(proto: Value) -> Self {
        Self {
            properties: HashMap::new(),
            prototype: proto.into(),
            array: Vec::new(),
        }
    }

    pub fn array_position(&self, index: usize) -> (usize, bool) {
        if self.array.is_empty() {
            return (0, false);
        }
        
        if self.array.len() > 100 {
            return self
                .array
                .binary_search_by(|(i, _)| i.cmp(&index)).map_or_else(|i| (i, false), |i| (i, true));
        }

        for (i, (j, _)) in self.array.iter().enumerate() {
            if *j == index {
                return (i, true);
            }

            if *j > index {
                return (i, false);
            }
        }

        (self.array.len(), false)
    }

    pub fn insert_array(&mut self, index: usize, value: Variable) {
        let (i, found) = self.array_position(index);

        if found {
            if let Some(v) = self.array.get_mut(i) {
                v.1 = value;
                return;
            };
        }

        self.array.insert(i, (index, value));
    }

    pub fn resolve_array(&self, index: usize) -> Option<Value> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get(i).map(|v| v.1.copy());
        }

        None
    }

    pub fn get_array(&self, index: usize) -> Option<&Value> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get(i).map(|v| &v.1.value);
        }

        None
    }

    pub fn get_array_mut(&mut self, index: usize) -> Option<&mut Value> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get_mut(i).map(|v| &mut v.1.value); //TODO: Check for perms
        }

        None
    }

    pub fn contains_array_key(&self, index: usize) -> bool {
        let (_, found) = self.array_position(index);

        found
    }

    pub fn from_values(values: Vec<(Value, Value)>, ctx: &mut Context) -> Self {
        let mut object = Self::raw(ctx);

        for (key, value) in values {
            object.define_property(key, value);
        }

        object
    }
}

impl Obj<Context> for Object {
    fn define_property(&mut self, name: Value, value: Value) {
        if let Value::Number(n) = &name {
            self.insert_array(*n as usize, value.into());
            return;
        }

        self.properties.insert(name, value.into());
    }

    fn define_variable(&mut self, name: Value, value: Variable) {
        if let Value::Number(n) = &name {
            self.insert_array(*n as usize, value);
            return;
        }
        self.properties.insert(name, value);
    }

    fn resolve_property(&self, name: &Value) -> Option<Value> {
        if name == &Value::String("__proto__".to_string()) {
            return Some(self.prototype.copy());
        }

        if let Value::Number(n) = name {
            return self.resolve_array(*n as usize);
        }

        self.properties
            .get(name)
            .map(yavashark_value::variable::Variable::copy)
            .or_else(|| match &self.prototype.value {
                Value::Object(o) => o.get_property(name).ok(),
                Value::Function(f) => f.get_property(name).ok(),
                _ => None,
            })
    }

    fn get_property(&self, name: &Value) -> Option<&Value> {
        if name == &Value::String("__proto__".to_string()) {
            return Some(&self.prototype.value);
        }

        if let Value::Number(n) = name {
            return self.get_array(*n as usize);
        }

        Some(&self.properties.get(name)?.value)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        if name == &Value::String("__proto__".to_string()) {
            return Some(&mut self.prototype.value);
        }

        if let Value::Number(n) = name {
            return self.get_array_mut(*n as usize);
        }

        Some(&mut self.properties.get_mut(name)?.value)
    }

    fn contains_key(&self, name: &Value) -> bool {
        if name == &Value::String("__proto__".to_string()) {
            return true;
        }

        if let Value::Number(n) = name {
            return self.contains_array_key(*n as usize);
        }

        self.properties.contains_key(name)
    }

    fn name(&self) -> String {
        "Object".to_string()
    }

    fn to_string(&self) -> String {
        "[object Object]".to_string()
    }

    fn properties(&self) -> Vec<(Value, Value)> {
        self.array
            .iter()
            .map(|(i, v)| (Value::Number(*i as f64), v.copy()))
            .chain(self.properties.iter().map(|(k, v)| (k.copy(), v.copy())))
            .collect()
    }

    fn keys(&self) -> Vec<Value> {
        self.array
            .iter()
            .map(|(i, _)| Value::Number(*i as f64))
            .chain(self.properties.keys().map(yavashark_value::Value::copy))
            .collect()
    }

    fn values(&self) -> Vec<Value> {
        self.array
            .iter()
            .map(|(_, v)| v.copy())
            .chain(self.properties.values().map(yavashark_value::variable::Variable::copy))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::Value;

    #[test]
    fn object_creation_with_proto() {
        let mut context = Context::new().unwrap();
        let proto = Value::Number(42.0);
        let object = Object::with_proto(proto);

        // assert_eq!(obj.prototype.value, proto); //TODO: Add a function "get_proto" to Object
    }

    #[test]
    fn object_creation_raw_with_proto() {
        let proto = Value::Number(42.0);
        let object = Object::raw_with_proto(proto.copy());

        assert_eq!(object.prototype.value, proto);
    }

    #[test]
    fn array_position_empty_array() {
        let mut context = Context::new().unwrap();
        let object = Object::raw(&mut context);

        let (index, found) = object.array_position(0);

        assert_eq!(index, 0);
        assert!(!found);
    }

    #[test]
    fn array_position_non_empty_array() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.insert_array(0, Value::Number(42.0).into());

        let (index, found) = object.array_position(0);

        assert_eq!(index, 0);
        assert!(found);
    }

    #[test]
    fn insert_array() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.insert_array(0, Value::Number(42.0).into());

        assert_eq!(object.array[0].1.value, Value::Number(42.0));
    }

    #[test]
    fn resolve_array() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.insert_array(0, Value::Number(42.0).into());

        let value = object.resolve_array(0);

        assert_eq!(value, Some(Value::Number(42.0)));
    }

    #[test]
    fn get_array() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.insert_array(0, Value::Number(42.0).into());

        let value = object.get_array(0);

        assert_eq!(value, Some(&Value::Number(42.0)));
    }

    #[test]
    fn get_array_mut() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.insert_array(0, Value::Number(42.0).into());

        let value = object.get_array_mut(0);

        assert_eq!(value, Some(&mut Value::Number(42.0)));
    }

    #[test]
    fn contains_array_key() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.insert_array(0, Value::Number(42.0).into());

        let contains = object.contains_array_key(0);

        assert!(contains);
    }

    #[test]
    fn define_property() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.define_property(Value::String("key".to_string()), Value::Number(42.0));

        assert_eq!(
            object
                .properties
                .get(&Value::String("key".to_string()))
                .unwrap()
                .value,
            Value::Number(42.0)
        );
    }

    #[test]
    fn resolve_property() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.define_property(Value::String("key".to_string()), Value::Number(42.0));

        let value = object.resolve_property(&Value::String("key".to_string()));

        assert_eq!(value, Some(Value::Number(42.0)));
    }

    #[test]
    fn get_property() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.define_property(Value::String("key".to_string()), Value::Number(42.0));

        let value = object.get_property(&Value::String("key".to_string()));

        assert_eq!(value, Some(&Value::Number(42.0)));
    }

    #[test]
    fn get_property_mut() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.define_property(Value::String("key".to_string()), Value::Number(42.0));

        let value = object.get_property_mut(&Value::String("key".to_string()));

        assert_eq!(value, Some(&mut Value::Number(42.0)));
    }

    #[test]
    fn contains_key() {
        let mut context = Context::new().unwrap();
        let mut object = Object::raw(&mut context);
        object.define_property(Value::String("key".to_string()), Value::Number(42.0));

        let contains = object.contains_key(&Value::String("key".to_string()));

        assert!(contains);
    }
}
