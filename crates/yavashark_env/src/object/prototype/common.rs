#![allow(clippy::needless_pass_by_value, unused)]

use crate::realm::Realm;
use crate::{Error, Object, Value, ValueResult};

pub fn define_getter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }

    let name = args[0].copy();
    let get = args[1].copy();

    let this = this.as_object()?;

    this.define_getter(name, get)?;

    Ok(Value::Undefined)
}

pub fn define_setter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }

    let name = args[0].copy();
    let set = args[1].copy();

    let this = this.as_object()?;

    this.define_setter(name, set)?;

    Ok(Value::Undefined)
}

pub fn lookup_getter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let name = &args[0];

    let this = this.as_object()?;

    let getter = this.get_getter(name)?;

    Ok(getter.unwrap_or(Value::Undefined))
}

pub fn lookup_setter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let name = &args[0];

    let this = this.as_object()?;

    let setter = this.get_setter(name)?;

    Ok(setter.unwrap_or(Value::Undefined))
}

pub fn object_constructor(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    todo!("Object constructor")
}

pub fn has_own_property(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Ok(Value::Boolean(false));
    }

    let Value::Object(obj) = this else {
        return Ok(Value::Boolean(false));
    };

    let key = &args[0];

    Ok(obj.contains_key(key)?.into())
}

pub fn get_own_property_descriptor(
    args: Vec<Value>,
    this: Value,
    realm: &mut Realm,
) -> ValueResult {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }

    let Value::Object(obj) = &args[0] else {
        return Ok(Value::Undefined);
    };

    let key = &args[1];

    let obj = obj.get();

    let Some(prop) = obj.get_property(key)? else {
        return Ok(Value::Undefined);
    };

    let desc = Object::new(realm);

    prop.descriptor(&desc)?;

    Ok(desc.into())
}

pub fn is_prototype_of(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let mut search = args[0].copy();

    loop {
        let mut obj = search.as_object()?;
        let proto = obj.prototype()?;
        let proto = proto.get(search.clone(), realm)?;

        if proto.is_nullish() {
            return Ok(false.into());
        }

        if proto == this {
            return Ok(true.into());
        }

        search = proto;
    }
}

pub fn property_is_enumerable(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Ok(false.into())
}

pub fn to_locale_string(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Err(Error::new("Not implemented"))
}

pub fn to_string(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Ok("[object Object]".into())
}

pub fn value_of(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Ok(this)
}
