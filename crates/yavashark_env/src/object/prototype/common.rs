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

    Ok(this
        .resolve_property_no_get_set(name)?
        .map_or(Value::Undefined, |p| p.get))
}

pub fn lookup_setter(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if args.is_empty() {
        return Ok(Value::Undefined);
    }

    let name = &args[0];

    let this = this.as_object()?;

    Ok(this
        .resolve_property_no_get_set(name)?
        .map_or(Value::Undefined, |p| p.set))
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

    let obj = obj.guard();

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

    let Value::Object(mut v) = args[0].copy() else {
        return Ok(false.into());
    };

    let o = this.to_object()?;

    loop {
        let proto = v.prototype()?;
        let proto = proto.get(v.clone().into(), realm)?;

        let Value::Object(proto) = proto else {
            return Ok(false.into());
        };

        v = proto;

        if v == o {
            return Ok(true.into());
        }
    }
}

pub fn property_is_enumerable(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    let Some(prop) = args.first() else {
        return Ok(false.into());
    };

    let Value::Object(obj) = this else {
        return Ok(false.into());
    };

    let prop = obj.get_property(prop)?;

    Ok(prop.attributes.is_enumerable().into())
}

pub fn to_locale_string(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Err(Error::new("Not implemented"))
}

pub fn to_string(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    if let Value::Object(this) = this {
        return Ok(format!("[object {}]", this.name()).into());
    }

    Ok(format!("[object {}]", this.ty()).into())
}

pub fn value_of(args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
    Ok(this)
}
