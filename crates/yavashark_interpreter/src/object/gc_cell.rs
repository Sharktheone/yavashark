use std::ops::{Deref, DerefMut};
use crate::{Value, Variable, ObjectHandle};

#[derive(Debug, PartialEq, Eq)]
pub struct ObjectVariable {
    value: Variable
}

pub struct RefMut<'a> {
    value: &'a mut Value,
    parent: ObjectHandle,
}

impl DerefMut for RefMut<'_> {
    fn deref_mut(&mut self) -> &mut Value {
        self.value
    }
}

impl Deref for RefMut<'_> {
    type Target = Value;
    
    fn deref(&self) -> &Value {
        self.value
    }
}


impl Drop for RefMut<'_> {
    fn drop(&mut self) {
        match self.value {
            Value::Object(obj) => {
                self.parent.0.add_ref(&obj.0);
            }
            Value::Function(func) => {
                self.parent.0.add_ref(&func.0);
            }
            _ => {}
        }
    }
}

impl ObjectVariable {
    const fn new(value: Variable) -> Self {
        Self { value }
    }
    
    fn new_value(value: Value) -> Self {
        Self { value: Variable::new(value) }
    }
    
    fn get(&self) -> &Variable {
        &self.value
    }
    
    fn replace(&mut self, value: Value, parent: ObjectHandle) {
        self.value = Variable::new(value);
        
        match &self.value.value {
            Value::Object(obj) => {
                parent.0.add_ref(&obj.0);
            }
            Value::Function(func) => {
                parent.0.add_ref(&func.0);
            }
            _ => {}
        }
    }
    
    fn get_mut(&mut self, parent: ObjectHandle) -> RefMut {
        match &self.value.value {
            Value::Object(obj) => {
                parent.0.remove_ref(&obj.0);
            }
            Value::Function(func) => {
                parent.0.remove_ref(&func.0); //TODO: maybe there is a better way, so we only remove the references if the object is not the same?
            }
            _ => {}
        }
        
        
        RefMut {
            value: &mut self.value.value,
            parent
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObjectValue {
    value: Value
}

impl ObjectValue {
    const fn new(value: Value) -> Self {
        Self { value }
    }
    
    fn get(&self) -> &Value {
        &self.value
    }
    
    fn replace(&mut self, value: Value, parent: ObjectHandle) {
        self.value = value;
        
        match &self.value {
            Value::Object(obj) => {
                parent.0.add_ref(&obj.0);
            }
            Value::Function(func) => {
                parent.0.add_ref(&func.0);
            }
            _ => {}
        }
    }
    
    fn get_mut(&mut self, parent: ObjectHandle) -> RefMut {
        match &self.value {
            Value::Object(obj) => {
                parent.0.remove_ref(&obj.0);
            }
            Value::Function(func) => {
                parent.0.remove_ref(&func.0); //TODO: maybe there is a better way, so we only remove the references if the object is not the same?
            }
            _ => {}
        }
        
        
        RefMut {
            value: &mut self.value,
            parent
        }
    }
}







