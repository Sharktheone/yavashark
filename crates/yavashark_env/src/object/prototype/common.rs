#![allow(clippy::needless_pass_by_value, unused)]

use crate::realm::Realm;
use crate::{Object, Value, ValueResult};

pub fn define_getter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!()
}

pub fn define_setter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!()
}

pub fn lookup_getter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!()
}

pub fn lookup_setter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!()
}

pub fn object_constructor(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!()
}

pub fn has_own_property(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.len() < 2{
        return Ok(Value::Undefined);
    }


    let Value::Object(obj) = &args[0] else {
        return Ok(Value::Undefined);
    };

    let key = &args[1];
    
    let obj = obj.get()?;

    Ok(obj.contains_key(key).into())
}

pub fn get_own_property_descriptor(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.len() < 2{
        return Ok(Value::Undefined);
    }
    
    
    let Value::Object(obj) = &args[0] else {
        return Ok(Value::Undefined);
    };
    
    let key = &args[1];
    
    let obj = obj.get()?;
    
    let Some(prop) = obj.resolve_property(key) else {
        return Ok(Value::Undefined);
    };
    
    let desc = Object::new(realm);
    
    prop.descriptor(desc.clone())?;
    
    
    Ok(desc.into())
}

pub fn is_prototype_of(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!()
}

pub fn property_is_enumerable(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!()
}

pub fn to_locale_string(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!()
}

pub fn to_string(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Ok(this.to_string(realm)?.into())
}

pub fn value_of(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!()
}

